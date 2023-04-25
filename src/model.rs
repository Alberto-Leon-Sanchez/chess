use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    time::{SystemTime, UNIX_EPOCH},
    vec,
};

use crate::{api::{board120_to_board64, board64_to_board120}, alpha_beta_search::{alpha_beta_min_net, alpha_beta_min, alpha_beta_max}};
use crate::attack_gen;
use crate::{
    alpha_beta_search,
    fen_reader,
    game::{self, GameInfo},
    make_move, move_gen,
    piece::{PieceList},
    suite,
    unmake::{self},
    eval
};
use rand::{Rng, SeedableRng};
use tch::{
    nn::{self, Module, OptimizerConfig, Sequential, ConvConfig, ModuleT},
    Tensor,
};

const N_STEPS: i64 = 12;
const N_EPOCHS: i64 = 1000000;
const N_GAMES: i64 = 256;
const LAMBDA: f64 = 0.7;
const MAX_NOT_IMPROVED: i64 = 80;
const DEPTH: i8 = 2;
const UNINITIALIZED: f64 = 100.00;
const LR: f64 = 0.001;

#[derive(Debug)]
pub struct Net {
    piece_pos: tch::nn::Linear,
    game_state: tch::nn::Linear,
    attacks: tch::nn::Linear,
    hidden1: tch::nn::Linear,
    hidden2: tch::nn::Linear,
    hidden3: tch::nn::Linear,
    hidden4: tch::nn::Linear,
}

impl Module for Net {
    fn forward(&self, xs: &tch::Tensor) -> tch::Tensor {
        let piece_pos = self.piece_pos.forward(&xs.slice(0, 0, 384, 1)).relu();
        let game_state = self.game_state.forward(&xs.slice(0, 384, 402, 1)).relu();
        let attacks = self.attacks.forward(&xs.slice(0, 402, 530, 1)).relu();

        let cat = Tensor::cat(&[piece_pos, game_state, attacks], 0);

        let hidden1 = self.hidden1.forward(&cat).relu();
        let hidden2 = self.hidden2.forward(&hidden1).relu();
        let hidden3 = self.hidden3.forward(&hidden2).relu();
        let result = self.hidden4.forward(&hidden3).tanh();

        result
    }
}

pub fn pre_proccess(game: &mut game::GameInfo) -> tch::Tensor {
    let bitmaps =
        piece_lists_to_bitmaps(&game.white_pieces, &game.black_pieces).totype(tch::Kind::Float);
    let game_state = game_state(game).totype(tch::Kind::Float);
    let attacks = attacks(game).totype(tch::Kind::Float);

    Tensor::cat(&[bitmaps, game_state, attacks], 0).totype(tch::Kind::Float)
}

pub fn model(vs: nn::Path) -> Net {
    let piece_pos = tch::nn::linear(&vs, 384, 384, Default::default());
    let game_state = tch::nn::linear(&vs, 18, 18, Default::default());
    let attacks = tch::nn::linear(&vs, 128, 128, Default::default());
    let hidden1 = tch::nn::linear(&vs, 530, 400, Default::default());
    let hidden2 = tch::nn::linear(&vs, 400, 200, Default::default());
    let hidden3 = tch::nn::linear(&vs, 200, 100, Default::default());
    let hidden4 = tch::nn::linear(&vs, 100, 1, Default::default());

    Net {
        piece_pos,
        game_state,
        attacks,
        hidden1,
        hidden2,
        hidden3,
        hidden4,
    }
}

fn get_training_games() -> Vec<GameInfo> {
    let mut games = vec![];

    let file = File::open("./training_fen.txt").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines(){
        games.push(fen_reader::read_fen(&line.unwrap()));
    }

    games
}

pub fn train() -> () {
    let mut vs = nn::VarStore::new(tch::Device::Cpu);
    let net = model(vs.root());

    let mut opt = nn::Adam::default().build(&vs, LR).unwrap();

    let mut suites = suite::get_suites();

    //vs.load_from_stream(&mut BufReader::new(File::open("./model_weights/bootstraping_2_hidden1425.pt").unwrap())).unwrap();
    
    opt.zero_grad();

    let mut games = get_training_games();

    let mut writer = OpenOptions::new()
        .append(true)
        .open("training_data/bootstraping_2_hidden.txt")
        .unwrap();

    for epoch in 0..N_EPOCHS {
        let mut accumulated_loss = 0.0;

        //tdl_train(&mut games, &net, &mut accumulated_loss);
        opt.zero_grad(); 
        bootstraping(&mut games, &net, &mut accumulated_loss);

        opt.step();
        opt.zero_grad();
        
        println!("Epoch: {} Loss: {}", epoch, accumulated_loss);
        if epoch % 5 == 0{
            writer
            .write_all(format!("{} {}\n", epoch, accumulated_loss).as_bytes())
            .unwrap();
        
            let score = suite::test_model_net(&net,&mut suites, epoch);
            println!("Epoch: {} Score: {}", epoch, score);

            vs.save(format!("model_weights/bootstraping_2_hidden{}.pt", epoch))
            .unwrap();
        }
    }
}

fn tdl_train(games: &mut Vec<GameInfo>, net: &Net, accumulated_loss: &mut f64) -> () {
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.try_into().unwrap());
    let size = games.len();
    let mut total_loss = tch::Tensor::of_slice(&[0.0]).set_requires_grad(true);
    for _ in 0..N_GAMES {
        let game = &mut games[rng.gen_range(0..size)].clone();
        let mut movements = move_gen::move_gen(game);
        let len = movements.len();

        if len == 0 {
            continue;
        }

        make_move::make_move(game, &mut movements[rng.gen_range(0..len)]);

        let mut scores = Vec::new();
        for _ in 0..=N_STEPS {
            
            match eval::is_game_over(game){
                Some(score) => {
                    scores.push(Tensor::of_slice(&[score]));
                    break;
                },
                None => (),
            }

            let mut movement = alpha_beta_search::best_move_net(DEPTH, game, net);
            make_move::make_move(game, &mut movement);

            let score = net.forward_t(&pre_proccess(game),true);
            scores.push(score);
        }

        let loss = get_loss(&scores);
        *accumulated_loss += loss.f_double_value(&[0]).unwrap();
        total_loss = total_loss + loss;
    }

    total_loss.backward();
}

fn get_loss(scores: &Vec<Tensor>) -> Tensor {
    let mut total_loss = Tensor::of_slice(&[0.0]);
    for i in 1..scores.len() {
        let step = i as f64;
        let discount_factor = tch::Tensor::of_slice(&[LAMBDA]).pow(&tch::Tensor::of_slice(&[step]));
        let current_loss = (scores.get(i).unwrap() - scores.get(i-1).unwrap()).multiply(&discount_factor);
        total_loss = total_loss + current_loss;
    }
    total_loss.abs()
}



fn bootstraping(
    games: &mut Vec<GameInfo>,
    net: &Net,
    accumulated_loss: &mut f64,
) -> () {

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.try_into().unwrap());
    let len = games.len();
    let mut losses = Vec::with_capacity(N_GAMES.try_into().unwrap());

    for _ in 0..N_GAMES {
        let game = &mut games[rng.gen_range(0..len)].clone();
        
        let mut movements = move_gen::move_gen(game);
        let len = movements.len();

        if len == 0 {
            continue;
        }

        let mut score = if game.turn == game::Color::White {
            tch::Tensor::of_slice(&[alpha_beta_search::alpha_beta_max(-1.0, 1.0, DEPTH, game)])
        }else{
            tch::Tensor::of_slice(&[alpha_beta_search::alpha_beta_min(-1.0, 1.0, DEPTH, game)])
        };
        let mut prediction = net.forward_t(&pre_proccess(game), true);

        let loss = get_loss_mse(&prediction, &score);
        *accumulated_loss += loss.f_double_value(&[0]).unwrap();
        losses.push(loss);
    }
    let len = losses.len();
    let sum_loss = losses.iter().fold(
        Tensor::of_slice(&[0.0]),
        |acc, x| acc + x,
    ) / tch::Tensor::of_slice(&[len as f64]);
    
    sum_loss.backward(); 
}
            
fn get_loss_mse(next_score: &Tensor, score: &Tensor) -> Tensor {
    
    (next_score - score).pow(&tch::Tensor::of_slice(&[2]))
}


fn piece_lists_to_bitmaps(white: &PieceList, black: &PieceList) -> tch::Tensor {
    let to_bitmap = |white: &Vec<i8>, black: &Vec<i8>| -> Tensor {
        let mut bitmap: Vec<i64> = vec![0; 64];

        for pos in white {
            bitmap[board120_to_board64(*pos) as usize] = 1;
        }

        for pos in black {
            bitmap[board120_to_board64(*pos) as usize] = -1;
        }

        Tensor::of_slice(&bitmap)
    };

    

    let pawns = to_bitmap(&white.pawns, &black.pawns);
    let knights = to_bitmap(&white.knights, &black.knights);
    let bishops = to_bitmap(&white.bishops, &black.bishops);
    let rooks = to_bitmap(&white.rooks, &black.rooks);
    let queens = to_bitmap(&white.queens, &black.queens);
    let kings = to_bitmap(&white.kings, &black.kings);

    Tensor::cat(&[pawns, knights, bishops, rooks, queens, kings], 0)
}

fn game_state(game: &GameInfo) -> tch::Tensor {
    let mut game_state: Vec<i64> = vec![0; 18];

    game_state[0] = game.white_pieces.pawns.len() as i64;
    game_state[1] = game.white_pieces.knights.len() as i64;
    game_state[2] = game.white_pieces.bishops.len() as i64;
    game_state[3] = game.white_pieces.rooks.len() as i64;
    game_state[4] = game.white_pieces.queens.len() as i64;
    game_state[5] = game.white_pieces.kings.len() as i64;

    game_state[6] = game.black_pieces.pawns.len() as i64;
    game_state[7] = game.black_pieces.knights.len() as i64;
    game_state[8] = game.black_pieces.bishops.len() as i64;
    game_state[9] = game.black_pieces.rooks.len() as i64;
    game_state[10] = game.black_pieces.queens.len() as i64;
    game_state[11] = game.black_pieces.kings.len() as i64;


    let castling = game.castling.last().unwrap();

    if castling[0] {
        game_state[12] = 1;
    }
    if castling[1] {
        game_state[13] = 1;
    }
    if castling[2] {
        game_state[14] = 1;
    }
    if castling[3] {
        game_state[15] = 1;
    }

    match game.turn {
        game::Color::White => game_state[16] = 1,
        game::Color::Black => game_state[17] = -1,
    }

    Tensor::of_slice(&game_state)
}

fn attacks(game: &mut GameInfo) -> tch::Tensor {
    let mut attacks: Vec<i64> = vec![0; 128];

    let (white_attacks, _) = attack_gen::attack_gen(game, Some(&game::Color::White));
    let (black_attacks, _) = attack_gen::attack_gen(game, Some(&game::Color::Black));

    let board120_to64 = |board: [u8; 120]| -> [u8; 64] {
        let mut board64: [u8; 64] = [0; 64];
        for i in 0..64 {
            board64[i] = board[board64_to_board120(i as i8) as usize];
        }
        board64
    };

    let white_attacks = board120_to64(white_attacks);
    let black_attacks = board120_to64(black_attacks);

    for i in 0..64 {
        if white_attacks[i] > 0 {
            attacks[i] = white_attacks[i] as i64;
        }
        if black_attacks[i] > 0 {
            attacks[i + 64] = black_attacks[i] as i64 * -1;
        }
    }

    Tensor::of_slice(&attacks)
}
