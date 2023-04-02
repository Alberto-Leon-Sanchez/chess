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
const N_GAMES: i64 = 128;
const LAMBDA: f64 = 0.7;
const MAX_NOT_IMPROVED: i64 = 80;
const DEPTH: i8 = 4;
const UNINITIALIZED: f64 = 100.00;

#[derive(Debug)]
pub struct Net {
    piece_pos: tch::nn::Linear,
    game_state: tch::nn::Linear,
    attacks: tch::nn::Linear,
    hidden1: tch::nn::Linear,
}

pub fn pre_proccess(game: &mut game::GameInfo) -> tch::Tensor {
    let bitmaps =
        piece_lists_to_bitmaps(&game.white_pieces, &game.black_pieces).totype(tch::Kind::Float);
    let game_state = game_state(game).totype(tch::Kind::Float);
    let attacks = attacks(game).totype(tch::Kind::Float);

    let input = Tensor::cat(&[bitmaps, game_state, attacks], 0);
    
    input.view([1, 1, 965])
}

#[derive(Debug)]
pub struct ChessCNN {
    layers: Sequential,
}

impl ChessCNN {
    pub fn new(vs: &nn::Path) -> Self {
        let cfg = nn::ConvConfigND{
            stride: 1,
            padding: 1,
            ..Default::default()
        };

        let layers = nn::seq()
        .add(nn::conv1d(vs, 1, 32, 3, cfg))
        .add_fn(|xs| xs.relu())
        .add(nn::conv1d(vs, 32, 64, 3, cfg))
        .add_fn(|xs| xs.relu())
        .add_fn(|xs| xs.view([-1, 64 * 965]))
        .add(nn::linear(vs, 64 * 965, 256, Default::default()))
        .add_fn(|xs| xs.relu())
        .add(nn::linear(vs, 256, 1, Default::default()))
        .add_fn(|xs| xs.tanh());

        ChessCNN { layers }
    }
}

impl nn::ModuleT for ChessCNN {
    fn forward_t(&self, xs: &Tensor, train: bool) -> Tensor {
        self.layers.forward_t(xs, train)
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
    let net = ChessCNN::new(&vs.root());

    //let mut opt = nn::Sgd::default().build(&vs, 0.00005).unwrap();

    let mut opt = nn::Adam::default().build(&vs, 0.001).unwrap();

    let mut suites = suite::get_suites();

    //vs.load_from_stream(&mut BufReader::new(File::open("./model_weights/2_hidden18585.pt").unwrap())).unwrap();
    
    opt.zero_grad();

    let mut games = get_training_games();

    let mut writer = OpenOptions::new()
        .append(true)
        .open("training_data/2_hidden.txt")
        .unwrap();

    for epoch in 18586..N_EPOCHS {
        let mut accumulated_loss = 0.0;

        //tdl_train(&mut games, &net, &mut accumulated_loss);
        
        bootstraping(&mut games, &net, &mut accumulated_loss);
        let score = suite::test_model_net(&net,&mut suites);

        opt.step();
        opt.zero_grad();
        
        println!("Epoch: {} Loss: {}", epoch, accumulated_loss);
        writer
            .write_all(format!("{} {}\n", epoch, accumulated_loss).as_bytes())
            .unwrap();

        println!("Epoch: {} Score: {}", epoch, score);

        vs.save(format!("model_weights/2_hidden{}.pt", epoch))
            .unwrap();
        
    }
}

fn tdl_train(games: &mut Vec<GameInfo>, net: &ChessCNN, accumulated_loss: &mut f64) -> () {
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.try_into().unwrap());
    let mut losses = Vec::new();
    let size = games.len();

    for _ in 0..N_GAMES {
        let game = &mut games[rng.gen_range(0..size)];
        let mut movements = move_gen::move_gen(game);
        let len = movements.len();

        if len == 0 {
            continue;
        }

        make_move::make_move(game, &mut movements[rng.gen_range(0..len)]);

        for step in 0..=N_STEPS {
            if move_gen::move_gen(game).len() == 0 {
                break;
            }

            let mut movement = alpha_beta_search::best_move_net(DEPTH, game, net);
            let actual_score = net.forward_t(&pre_proccess(game), true);

            make_move::make_move(game, &mut movement);
            let loss = get_loss(&net.forward_t(&pre_proccess(game), true), &actual_score, step); 

            *accumulated_loss += loss.f_double_value(&[0]).unwrap();
            losses.push(loss);

        }
    }

    let sum_loss = losses.iter().fold(Tensor::of_slice(&[0.0]), |acc, x| acc + x);
    let mean_loss = sum_loss / (N_GAMES as f64);
    *accumulated_loss /= (N_GAMES as f64);
    mean_loss.backward();
}  

fn bootstraping(
    games: &mut Vec<GameInfo>,
    net: &ChessCNN,
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
        let game = &mut games[rng.gen_range(0..len)];
        let mut movements = move_gen::move_gen(game);
        let len = movements.len();

        if len == 0{
            continue;
        }

        make_move::make_move(game, &mut movements[rng.gen_range(0..len)]);

        let mut score = if game.turn == game::Color::White {
            tch::Tensor::of_slice(&[alpha_beta_search::alpha_beta_max(-1.0, 1.0, DEPTH, game)])
        }else{
            tch::Tensor::of_slice(&[alpha_beta_search::alpha_beta_min(-1.0, 1.0, DEPTH, game)])
        };
        let mut prediction = net.forward_t(&pre_proccess(game), true);

        let loss = get_loss_mse(&prediction, &score);
        println!("{},{}", prediction, score);
        *accumulated_loss += loss.f_double_value(&[0]).unwrap();
        losses.push(loss);
    }
    let sum_loss = losses.iter().fold(
        Tensor::of_slice(&[0.0]),
        |acc, x| acc + x,
    );

    let mean_loss = sum_loss / (N_GAMES as f64);
    *accumulated_loss /= N_GAMES as f64;
    mean_loss.backward(); 
}
            
  
fn get_loss(next_score: &Tensor, score: &Tensor, step: i64) -> Tensor {
    let discount_factor = tch::Tensor::of_slice(&[LAMBDA])
        .pow(&tch::Tensor::of_slice(&[step as f64]));
    
    (next_score - score).multiply(&discount_factor).abs()
}

fn get_loss_mse(next_score: &Tensor, score: &Tensor) -> Tensor {
    
    (next_score - score).pow(&tch::Tensor::of_slice(&[2]))
}


fn piece_lists_to_bitmaps(white: &PieceList, black: &PieceList) -> tch::Tensor {
    let to_bitmap = |piece_list: &Vec<i8>| -> Tensor {
        let mut bitmap: Vec<i64> = vec![0; 64];

        for pos in piece_list {
            bitmap[board120_to_board64(*pos) as usize] = 1;
        }

        Tensor::of_slice(&bitmap)
    };

    let white_pawns = to_bitmap(&white.pawns);
    let black_pawns = to_bitmap(&black.pawns);
    let white_knights = to_bitmap(&white.knights);
    let black_knights = to_bitmap(&black.knights);
    let white_bishops = to_bitmap(&white.bishops);
    let black_bishops = to_bitmap(&black.bishops);
    let white_rooks = to_bitmap(&white.rooks);
    let black_rooks = to_bitmap(&black.rooks);
    let white_queens = to_bitmap(&white.queens);
    let black_queens = to_bitmap(&black.queens);
    let white_kings = to_bitmap(&white.kings);
    let black_kings = to_bitmap(&black.kings);

    Tensor::cat(&[white_pawns,black_pawns,white_knights,black_knights,white_rooks,black_rooks,white_queens,black_queens,white_kings,black_kings, white_bishops, black_bishops], 0)
}

fn game_state(game: &GameInfo) -> tch::Tensor {
    let mut game_state: Vec<i64> = vec![0; 69];

    match game.en_passant.last().unwrap() {
        Some(pos) => game_state[board120_to_board64(*pos) as usize] = 1,
        None => (),
    }

    let castling = game.castling.last().unwrap();

    if castling[0] {
        game_state[64] = 1;
    }
    if castling[1] {
        game_state[65] = 1;
    }
    if castling[2] {
        game_state[66] = 1;
    }
    if castling[3] {
        game_state[67] = 1;
    }

    match game.turn {
        game::Color::White => game_state[68] = 1,
        _ => (),
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
            attacks[i + 64] = black_attacks[i] as i64;
        }
    }

    Tensor::of_slice(&attacks)
}

