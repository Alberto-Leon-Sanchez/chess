use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use regex;
use tch::nn::{self};

use crate::{
    eval, fen_reader, game::{self, GameInfo}, make_move, model,
    move_gen::{self, move_gen},
    notation, unmake,
};

const UNINITIALIZED: f64 = 10.0;

pub fn test_model() -> () {
    let paths: Vec<String> = fs::read_dir("./model_weights/")
        .unwrap()
        .into_iter()
        .map(|x| x.unwrap().path().to_str().unwrap().to_string())
        .collect();
    let mut vs = nn::VarStore::new(tch::Device::Cpu);
    let net = model::ChessCNN::new(&vs.root());
    let suites = get_suites();
    let mut games = suites.0;
    let results = suites.1;
    let mut total_score: i64;
    let mut suite_results = File::create("./suite_spike.txt").unwrap();

    for path in paths {
        vs.load_from_stream(&mut BufReader::new(File::open(&path).unwrap()))
            .unwrap();
        total_score = 0;

        for (mut game, result) in games.iter_mut().zip(results.iter()) {
            let moves = move_gen(game);
            let mut max: f64 = UNINITIALIZED;
            let mut best_move: move_gen::Move = move_gen::Move::new();

            for mut movement in moves {
                make_move::make_move(game, &mut movement);
                let score = eval::net_eval(&mut game, &net);

                if max == UNINITIALIZED {
                    max = score;
                    best_move = movement;
                }

                if game.turn == game::Color::White {
                    if score < max {
                        max = score;
                        best_move = movement;
                    }
                } else {
                    if score > max {
                        max = score;
                        best_move = movement;
                    }
                }

                unmake::unmake_move(game, movement);
            }

            for (movement, puntuaction) in result {
                if *movement == best_move {
                    total_score += puntuaction;
                    break;
                }
            }
        }
        suite_results
            .write_all(format!("{}: {}\n", path, total_score).as_bytes())
            .unwrap();
    }
}

pub fn test_model_net(net: &model::ChessCNN, suites: &mut (Vec<GameInfo>,Vec<Vec<(move_gen::Move,i64)>>)) -> i64 {
    let games = &mut suites.0;
    let results = &mut suites.1;
    let mut total_score: i64;
    let mut suite_results = OpenOptions::new().append(true).open("./suite.txt").unwrap();

    total_score = 0;

    for (mut game, result) in games.iter_mut().zip(results.iter()) {
        let moves = move_gen(game);
        let mut max: f64 = UNINITIALIZED;
        let mut best_move: move_gen::Move = move_gen::Move::new();

        for mut movement in moves {
            make_move::make_move(game, &mut movement);
            let score = eval::net_eval(&mut game, &net);

            if max == UNINITIALIZED {
                max = score;
                best_move = movement;
            }

            if game.turn == game::Color::White {
                if score < max {
                    max = score;
                    best_move = movement;
                }
            } else {
                if score > max {
                    max = score;
                    best_move = movement;
                }
            }

            unmake::unmake_move(game, movement);
        }

        for (movement, puntuaction) in result {
            if *movement == best_move {
                total_score += puntuaction;
                break;
            }
        }
    }
    suite_results
        .write_all(format!("{}\n", total_score).as_bytes())
        .unwrap();
    total_score
}

pub fn get_suites() -> (Vec<game::GameInfo>, Vec<Vec<(move_gen::Move, i64)>>) {
    let regex = regex::Regex::new("(([RBQKPN]?[1-8]?[a-h]?[x]?[a-h][1-8]([=][RBQKPN])?[+#]?=[0-9]*)?((O-O-O)?(O-O)?=[0-9]*)?)").unwrap();
    let paths: Vec<String> = fs::read_dir("./suites/")
        .unwrap()
        .into_iter()
        .map(|x| x.unwrap().path().to_str().unwrap().to_string())
        .collect();
    let mut games: Vec<game::GameInfo> = vec![];
    let mut results: Vec<Vec<(move_gen::Move, i64)>> = vec![];

    for path in paths {
        let file = File::open(path).unwrap();
        let buffer = BufReader::new(file);

        for line in buffer.lines() {
            let line = line
                .unwrap()
                .split("bm")
                .into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            let mut game = fen_reader::read_fen(&line[0]);
            let mut result: Vec<(move_gen::Move, i64)> = vec![];

            for capture in regex.captures_iter(&line[1]) {
                if capture.get(0).unwrap().as_str() != "" {
                    let tmp = capture
                        .get(0)
                        .unwrap()
                        .as_str()
                        .split("=")
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>();
                    let movement = notation::get_move(&tmp[0], &mut game);
                    let punctuation = tmp[1].parse::<i64>().unwrap();
                    result.push((movement, punctuation));
                }
            }

            games.push(game);
            results.push(result);
        }
    }

    (games, results)
}
