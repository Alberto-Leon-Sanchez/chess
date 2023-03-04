use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    time::{SystemTime, UNIX_EPOCH},
    vec,
};

use crate::api::{board120_to_board64, board64_to_board120};
use crate::attack_gen;
use crate::{
    alpha_beta_search,
    eval::{self, eval},
    fen_reader,
    game::{self, Eval, GameInfo},
    make_move, move_gen,
    piece::{Piece, PieceList},
    suite,
    unmake::{self, unmake_move},
};
use rand::{Rng, SeedableRng};
use tch::{
    nn::{self, Module, OptimizerConfig},
    Tensor,
};

const N_STEPS: i64 = 12;
const N_EPOCHS: i64 = 10000;
const N_GAMES: i64 = 256;
const LAMBDA: f64 = 0.7;
const BATCH_GAMES: i64 = 256;
const MAX_NOT_IMPROVED: i64 = 80;
const EVAl_PERCENTAGE: f64 = 0.0;
const DEPTH: i8 = 3;
const UNINITIALIZED: f64 = 100.00;

#[derive(Debug)]
pub struct Net {
    piece_pos: tch::nn::Linear,
    game_state: tch::nn::Linear,
    attacks: tch::nn::Linear,
    hidden1: tch::nn::Linear,
    hidden2: tch::nn::Linear,
}

impl Module for Net {
    fn forward(&self, xs: &tch::Tensor) -> tch::Tensor {
        let piece_pos = self.piece_pos.forward(&xs.slice(0, 0, 384, 1)).relu();
        let game_state = self.game_state.forward(&xs.slice(0, 384, 453, 1)).relu();
        let attacks = self.attacks.forward(&xs.slice(0, 453, 581, 1)).relu();

        let cat = Tensor::cat(&[piece_pos, game_state, attacks], 0);

        let hidden1 = self.hidden1.forward(&cat).relu();
        let result = self.hidden2.forward(&hidden1).tanh();

        result
    }
}

fn get_training_games() -> Vec<String> {
    let mut games = vec![];

    let file = File::open("./training_fen.txt").unwrap();
    let reader = BufReader::new(file);

    let num_games = reader.lines().count();

    let file = File::open("./training_fen.txt").unwrap();
    let reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        if i < num_games - (num_games as f64 * EVAl_PERCENTAGE).ceil() as usize {
            games.push(line.unwrap());
        }
    }

    games
}

fn get_eval_games() -> Vec<String> {
    let mut games = vec![];

    let file = File::open("./training_fen.txt").unwrap();
    let reader = BufReader::new(file);

    let num_games = reader.lines().count();

    let file = File::open("./training_fen.txt").unwrap();
    let reader = BufReader::new(file);

    //get the last num_games of the file
    for line in reader
        .lines()
        .skip(num_games - (num_games as f64 * EVAl_PERCENTAGE).ceil() as usize)
    {
        games.push(line.unwrap());
    }

    games
}

fn get_batch(games: &Vec<String>) -> Vec<game::GameInfo> {
    let mut batch: Vec<game::GameInfo> = vec![];

    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.try_into().unwrap());

    for _ in 0..BATCH_GAMES {
        batch.push(fen_reader::read_fen(&games[rng.gen_range(0..games.len())]));
    }
    batch
}

pub fn pre_proccess(game: &mut game::GameInfo) -> tch::Tensor {
    let bitmaps =
        piece_lists_to_bitmaps(&game.white_pieces, &game.black_pieces).totype(tch::Kind::Float);
    let game_state = game_state(game).totype(tch::Kind::Float);
    let attacks = attacks(game).totype(tch::Kind::Float);

    Tensor::cat(&[bitmaps, game_state, attacks], 0).totype(tch::Kind::Float)
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
        game::Color::White => game_state[68] = 10,
        game::Color::Black => game_state[68] = -10,
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

pub fn model(vs: nn::Path) -> Net {
    let piece_pos = tch::nn::linear(&vs, 384, 384, Default::default());
    let game_state = tch::nn::linear(&vs, 69, 69, Default::default());
    let attacks = tch::nn::linear(&vs, 128, 128, Default::default());
    let hidden1 = tch::nn::linear(&vs, 581, 581, Default::default());
    let hidden2 = tch::nn::linear(&vs, 581, 1, Default::default());

    Net {
        piece_pos,
        game_state,
        attacks,
        hidden1,
        hidden2,
    }
}

pub fn train() -> () {
    let mut vs = nn::VarStore::new(tch::Device::Cpu);
    let net = model(vs.root());
    let mut opt = nn::Adam::default().build(&vs, 0.001).unwrap();

    //vs.load_from_stream(&mut BufReader::new(File::open("./model_weights/3_hidden_1204.pt").unwrap())).unwrap();
    opt.zero_grad();

    let games = get_training_games();
    let num_batches: i64 = (N_GAMES as f64 / BATCH_GAMES as f64).ceil() as i64;

    let mut writer = OpenOptions::new()
        .append(true)
        .open("./training_data/bootstraping_3_hidden.txt")
        .unwrap();

    for epoch in 0..=N_EPOCHS {
        let mut accumulated_loss = 0.0;

        //tdl_train(num_batches, &games, &net, &mut accumulated_loss);
        bootstraping(num_batches, &games, &net, &mut accumulated_loss);
        let score = suite::test_model_net(&net);
        println!("{}", net.hidden1.ws);

        opt.step();
        opt.zero_grad();
        println!("{}", net.hidden1.ws);

        let batch_loss = accumulated_loss / num_batches as f64;

        println!("Epoch: {} Loss: {}", epoch, batch_loss);
        writer
            .write_all(format!("{} {}\n", epoch, batch_loss).as_bytes())
            .unwrap();

        println!("Epoch: {} Score: {}", epoch, score);

        vs.save(format!("model_weights/bootstraping_3_hidden_{}.pt", epoch))
            .unwrap();
    }
}

fn tdl_train(num_batches: i64, games: &Vec<String>, net: &Net, accumulated_loss: &mut f64) -> () {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.try_into().unwrap());

    for _ in 0..num_batches {
        let mut batch = get_batch(games);

        for game in batch.iter_mut() {
            let mut tmp = move_gen::move_gen(game);
            let len = tmp.len();
            make_move::make_move(game, &mut tmp[rng.gen_range(0..len)]);

            let mut last_prediction: tch::Tensor = Tensor::of_slice(&[0.0]);
            let mut losses = vec![];

            for step in 0..=N_STEPS {
                let mut moves = move_gen::move_gen(game);

                if moves.len() == 0 {
                    break;
                }

                let mut best_score: tch::Tensor;
                let mut best_move = moves[0];

                if game.turn == game::Color::White {
                    best_score = Tensor::of_slice(&[-1000000.0]);
                } else {
                    best_score = Tensor::of_slice(&[1000000.0]);
                }

                for movement in moves.iter_mut() {
                    make_move::make_move(game, movement);

                    let features = pre_proccess(game);
                    let prediction = net.forward(&features);

                    if game.turn == game::Color::White {
                        if prediction.f_double_value(&[0]).unwrap()
                            < best_score.f_double_value(&[0]).unwrap()
                        {
                            best_score = prediction;
                            best_move = *movement;
                        }
                    } else {
                        if prediction.f_double_value(&[0]).unwrap()
                            > best_score.f_double_value(&[0]).unwrap()
                        {
                            best_score = prediction;
                            best_move = *movement;
                        }
                    }

                    unmake::unmake_move(game, *movement);
                }

                let loss = get_loss(&best_score, &last_prediction, step);
                last_prediction = best_score;

                make_move::make_move(game, &mut best_move);

                *accumulated_loss += loss.f_double_value(&[0]).unwrap();
                losses.push(loss);

                //step bucle
            }
            //game bucle
            let sum_loss = losses.iter().fold(
                Tensor::of_slice(&[0.0]).set_requires_grad(true),
                |acc, x| acc + x,
            );
            sum_loss.backward();
        }
        //batch bucle
    }
}

fn bootstraping(
    num_batches: i64,
    games: &Vec<String>,
    net: &Net,
    accumulated_loss: &mut f64,
) -> () {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed.try_into().unwrap());

    for _ in 0..num_batches {
        let mut batch = get_batch(games);
        let mut losses = vec![];

        for game in batch.iter_mut() {
            let mut movements = move_gen::move_gen(game);
            let len = movements.len();
            make_move::make_move(game, &mut movements[rng.gen_range(0..len)]);

            movements = move_gen::move_gen(game);

            let mut prediction;
            let mut score;
            let mut score_selected = tch::Tensor::of_slice(&[0.0]);
            let mut best_prediction = if game.turn == game::Color::White {
                tch::Tensor::of_slice(&[f64::MIN])
            } else {
                tch::Tensor::of_slice(&[f64::MAX])
            };

            for mut movement in movements {
                make_move::make_move(game, &mut movement);
                prediction = alpha_beta_search::alpha_beta_min_net(
                    tch::Tensor::of_slice(&[f64::MIN]),
                    tch::Tensor::of_slice(&[f64::MAX]),
                    DEPTH - 1,
                    game,
                    net,
                );
                score = tch::Tensor::of_slice(&[alpha_beta_search::alpha_beta_min(
                    f64::MIN,
                    f64::MAX,
                    DEPTH - 1,
                    game,
                )]);
                unmake::unmake_move(game, movement);

                if game.turn == game::Color::White {
                    if prediction.f_double_value(&[0]).unwrap()
                        > best_prediction.f_double_value(&[0]).unwrap()
                    {
                        best_prediction = prediction;
                        score_selected = score;
                    }
                } else {
                    if prediction.f_double_value(&[0]).unwrap()
                        < best_prediction.f_double_value(&[0]).unwrap()
                    {
                        best_prediction = prediction;
                        score_selected = score;
                    }
                }
            }
            let loss = get_loss(&best_prediction, &score_selected, 0);
            println!("{},{}", best_prediction, score_selected);
            *accumulated_loss += loss.f_double_value(&[0]).unwrap();
            losses.push(loss);
        }
        let sum_loss = losses.iter().fold(
            Tensor::of_slice(&[0.0]).set_requires_grad(true),
            |acc, x| acc + x,
        );
        sum_loss.backward();
    }
}

fn get_loss(next_score: &Tensor, score: &Tensor, step: i64) -> Tensor {
    let discount_factor = tch::Tensor::of_slice(&[LAMBDA])
        .set_requires_grad(true)
        .pow(&tch::Tensor::of_slice(&[step as f64]).set_requires_grad(true));
    let error = (next_score - score).multiply(&discount_factor).abs();
    let loss = error.pow(&tch::Tensor::of_slice(&[2.0]).set_requires_grad(true));

    error
}
