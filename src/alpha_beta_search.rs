use crate::eval;
use crate::game;
use crate::make_move;
use crate::model;
use crate::move_gen;
use crate::piece;
use crate::unmake;

const UNINITIALIZED: f64 = 100.00;

static mut hash_access: u64 = 0;
static mut alpha_cuts: u64 = 0;
static mut beta_cuts: u64 = 0;

pub fn get_best(game: &mut game::GameInfo, depth: i8, net: &model::Net) -> move_gen::Move {
    let mut best_move = move_gen::Move {
        origin: 0,
        destiny: 0,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    let mut best_score: f64 = 100.00;

    let moves = move_gen::move_gen(game);

    for mut mov in moves {
        make_move::make_move(game, &mut mov);
        let score = alpha_beta_min_net(
            tch::Tensor::of_slice(&[f64::MIN]),
            tch::Tensor::of_slice(&[f64::MAX]),
            depth - 1,
            game,
            net,
        )
        .f_double_value(&[0])
        .unwrap();

        if best_score == UNINITIALIZED {
            best_score = score;
            best_move = mov
        }

        if score > best_score {
            best_score = score;
            best_move = mov;
        }

        unmake::unmake_move(game, mov);
    }

    println!("{},{}", unsafe { beta_cuts }, unsafe { alpha_cuts });

    best_move
}

fn order_moves(moves: &mut Vec<move_gen::Move>, scores: &Vec<f64>) {
    for i in 0..moves.len() - 1 {
        for j in i + 1..moves.len() {
            if scores[i] < scores[j] {
                moves.swap(i, j);
            }
        }
    }
}

pub fn alpha_beta_max_net(
    alpha: tch::Tensor,
    beta: tch::Tensor,
    depth: i8,
    game: &mut game::GameInfo,
    net: &model::Net,
) -> tch::Tensor {
    if depth == 0 {
        return eval::net_eval_tch(game, net);
    }

    let mut alpha = alpha;
    let movements = move_gen::move_gen(game);

    if movements.len() == 0 {
        if game.turn == game::Color::White {
            return tch::Tensor::of_slice(&[-1.0]);
        } else {
            return tch::Tensor::of_slice(&[1.0]);
        }
    }

    for mut movement in movements {
        make_move::make_move(game, &mut movement);
        let score = alpha_beta_min_net(alpha.copy(), beta.copy(), depth - 1, game, net);
        unmake::unmake_move(game, movement);

        if score.f_double_value(&[0]).unwrap() >= beta.f_double_value(&[0]).unwrap() {
            return beta;
        }

        if score.f_double_value(&[0]).unwrap() > alpha.f_double_value(&[0]).unwrap() {
            alpha = score;
        }
    }
    alpha
}

pub fn alpha_beta_min_net(
    alpha: tch::Tensor,
    beta: tch::Tensor,
    depth: i8,
    game: &mut game::GameInfo,
    net: &model::Net,
) -> tch::Tensor {
    if depth == 0 {
        return eval::net_eval_tch(game, net);
    }

    let mut beta = beta;
    let movements = move_gen::move_gen(game);

    if movements.len() == 0 {
        if game.turn == game::Color::White {
            return tch::Tensor::of_slice(&[-1.0]);
        } else {
            return tch::Tensor::of_slice(&[1.0]);
        }
    }

    for mut movement in movements {
        make_move::make_move(game, &mut movement);
        let score = alpha_beta_max_net(alpha.copy(), beta.copy(), depth - 1, game, net);
        unmake::unmake_move(game, movement);

        if score.f_double_value(&[0]).unwrap() <= alpha.f_double_value(&[0]).unwrap() {
            return alpha;
        }

        if score.f_double_value(&[0]).unwrap() < beta.f_double_value(&[0]).unwrap() {
            beta = score;
        }
    }
    beta
}

pub fn alpha_beta_max(alpha: f64, beta: f64, depth: i8, game: &mut game::GameInfo) -> f64 {
    if depth == 0 {
        return eval::eval(game);
    }

    let mut alpha = alpha;
    let movements = move_gen::move_gen(game);

    if movements.len() == 0 {
        if game.turn == game::Color::White {
            return -1.0;
        } else {
            return 1.0;
        }
    }

    for mut movement in movements {
        make_move::make_move(game, &mut movement);
        let score = alpha_beta_min(alpha, beta, depth - 1, game);
        unmake::unmake_move(game, movement);

        if score >= beta {
            return beta;
        }

        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

pub fn alpha_beta_min(alpha: f64, beta: f64, depth: i8, game: &mut game::GameInfo) -> f64 {
    if depth == 0 {
        return eval::eval(game);
    }

    let mut beta = beta;
    let movements = move_gen::move_gen(game);

    if movements.len() == 0 {
        if game.turn == game::Color::White {
            return -1.0;
        } else {
            return 1.0;
        }
    }

    for mut movement in movements {
        make_move::make_move(game, &mut movement);
        let score = alpha_beta_max(alpha, beta, depth - 1, game);
        unmake::unmake_move(game, movement);

        if score <= alpha {
            return alpha;
        }

        if score < beta {
            beta = score;
        }
    }
    beta
}
