use std::time::Duration;
use std::time::Instant;

use serde::__private::de;

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

pub fn alpha_beta_max_net(
    alpha: f64,
    beta: f64,
    depth_left: i8,
    game: &mut game::GameInfo,
    pv: &mut Vec<move_gen::Move>,
    start_time: &Instant,
    time_limit: Duration,
    net: &model::Net,
    ) -> f64 {
        
    let mut movements = move_gen::move_gen(game);
    
    if depth_left == 0 || movements.len() == 0 {
        return eval::net_eval(game, net);
    }
    
    let mut alpha = alpha;
    let mut beta = beta;
    
    let index: usize = (game.hash % game::TRANSPOSITION_TABLE_SIZE) as usize;
    if game.transposition_table[index].zobrist_key == game.hash && game.transposition_table[index].depth >= depth_left{
        match game.transposition_table[index].flag{
            game::Flag::Exact => return game.transposition_table[index].value,
            game::Flag::Lowerbound => alpha = alpha.max(game.transposition_table[index].value),
            game::Flag::Upperbound => beta = beta.min(game.transposition_table[index].value),
        }
        if alpha >= beta {
            return game.transposition_table[index].value;
        }
    }
    
    let mut new_pv: Vec<move_gen::Move> = Vec::new();
    let mut first = false;
    
    eval::order_moves(&mut movements, &pv, game,1);
    
    for mut movement in movements {
    
        if start_time.elapsed() >= time_limit {
            return -100.0;
        }
    
        make_move::make_move(game, &mut movement);
    
        let mut score;
        if first {
            score = alpha_beta_min_net(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, net);
            first = false;
        } else {
            score = alpha_beta_min_net(beta - 1.0, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, net);
            if score > alpha && score < beta {
                score = alpha_beta_min_net(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, net);
            }
        }
        unmake::unmake_move(game, movement);
    
        if score >= beta {
            return beta;
        }
    
        if score > alpha {
            alpha = score;
            pv.clear();
            pv.push(movement);
            pv.append(&mut new_pv);
        }
    }
    
    let flag = if alpha <= alpha {
        game::Flag::Upperbound
    } else if alpha >= beta {
        game::Flag::Lowerbound
    } else {
        game::Flag::Exact
    };
    
    let index = (game.hash % game::TRANSPOSITION_TABLE_SIZE) as usize;
    
    game.transposition_table[index].zobrist_key = game.hash;
    game.transposition_table[index].flag = flag;
    game.transposition_table[index].depth = depth_left;
    game.transposition_table[index].value = alpha;
    
    alpha
}

pub fn alpha_beta_min_net(
    alpha: f64,
    beta: f64,
    depth_left: i8,
    game: &mut game::GameInfo,
    pv: &mut Vec<move_gen::Move>,
    start_time: &Instant,
    time_limit: Duration,
    net: &model::Net,
    ) -> f64 {
        
    let mut movements = move_gen::move_gen(game);
    
    if depth_left == 0 || movements.len() == 0 {
        return eval::net_eval(game, net);
    }
    
    let mut alpha = alpha;
    let mut beta = beta;
    
    let index = (game.hash % game::TRANSPOSITION_TABLE_SIZE) as usize;
    if game.transposition_table[index].zobrist_key == game.hash && game.transposition_table[index].depth >= depth_left{
        match game.transposition_table[index].flag{
            game::Flag::Exact => return game.transposition_table[index].value,
            game::Flag::Lowerbound => alpha = alpha.max(game.transposition_table[index].value),
            game::Flag::Upperbound => beta = beta.min(game.transposition_table[index].value),
        }
        if alpha >= beta {
            return game.transposition_table[index].value;
        }
    }
    
    let mut new_pv: Vec<move_gen::Move> = Vec::new();
    let mut first = false;
    
    eval::order_moves(&mut movements, &pv, game, 1);
    
    for mut movement in movements {
    
        if start_time.elapsed() >= time_limit {
            return 100.0;
        }
    
        make_move::make_move(game, &mut movement);
    
        let mut score;
        if first {
            score = alpha_beta_max_net(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, net);
            first = false;
        } else {
            score = alpha_beta_max_net(alpha, beta - 1.0, depth_left - 1, game, &mut new_pv, start_time, time_limit, net);
            if score < beta && score > alpha {
                score = alpha_beta_max_net(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, net);
            }
        }
        unmake::unmake_move(game, movement);
    
        if score <= alpha {
            return alpha;
        }
    
        if score < beta {
            beta = score;
            pv.clear();
            pv.push(movement);
            pv.append(&mut new_pv);
        }
    }
    
    let flag = if beta <= alpha {
        game::Flag::Lowerbound
    } else if beta >= beta {
        game::Flag::Upperbound
    } else {
        game::Flag::Exact
    };
    
    let index = (game.hash % game::TRANSPOSITION_TABLE_SIZE) as usize;
    
    game.transposition_table[index].zobrist_key = game.hash;
    game.transposition_table[index].flag = flag;
    game.transposition_table[index].depth = depth_left;
    game.transposition_table[index].value = beta;
    
    beta
    
}
   

pub fn alpha_beta_max(alpha: f64, beta: f64, depth_left: i8, game: &mut game::GameInfo, pv: &mut Vec<move_gen::Move>,start_time: &Instant, time_limit: &Duration, ply: i8) -> f64 {

    let mut movements = move_gen::move_gen(game);

    if depth_left == 0 || movements.len() == 0 {
        return eval::static_evaluate(game);
    }

    let mut alpha = alpha;
    let mut beta = beta;
    
    let index = (game.hash % game::TRANSPOSITION_TABLE_SIZE) as usize;
    if game.transposition_table[index].zobrist_key == game.hash && game.transposition_table[index].depth >= depth_left{
        match game.transposition_table[index].flag{
            game::Flag::Exact => return game.transposition_table[index].value,
            game::Flag::Lowerbound => alpha = alpha.max(game.transposition_table[index].value),
            game::Flag::Upperbound => beta = beta.min(game.transposition_table[index].value),
        }
        if alpha >= beta {
            return game.transposition_table[index].value;
        }
    }
    
    let mut new_pv: Vec<move_gen::Move> = Vec::new();
    let mut first = false;
    let mut value = -100.0;

    eval::order_moves(&mut movements, &pv, game, ply);

    for mut movement in movements {

        if start_time.elapsed() >= *time_limit {
            return -100.0;
        }

        make_move::make_move(game, &mut movement);
        
        let mut score;
        if first{
            score = alpha_beta_min(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1);
            first = false;
        }else{
            score = alpha_beta_min(beta - 1.0, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1);
            if score > alpha && score < beta {
                score = alpha_beta_min(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1);
            }
        }
        unmake::unmake_move(game, movement);

        if score >= beta {
            if movement.destiny_piece == piece::Piece::Empty{

                let side = if game.turn == game::Color::White {0} else {1};
                game.historic_heuristic[side][movement.origin as usize][movement.destiny as usize] +=( depth_left * depth_left) as usize;
                
                if game.killer_move[ply as usize][0] != movement{
                    game.killer_move[ply as usize][1] = game.killer_move[ply as usize][0];
                    game.killer_move[ply as usize][0] = movement;
                }

            }

            return beta;
        }

        if score > alpha {
            alpha = score; 
            pv.clear();
            pv.push(movement);
            pv.append(&mut new_pv);
            value = score;
        }
    }
    
    if alpha <= alpha {
        game.transposition_table[index].flag = game::Flag::Upperbound;
    } else if alpha >= beta {
        game.transposition_table[index].flag = game::Flag::Lowerbound;
    } else {
        game.transposition_table[index].flag = game::Flag::Exact;
    }
    
    game.transposition_table[index].zobrist_key = game.hash;
    game.transposition_table[index].depth = depth_left;
    game.transposition_table[index].value = value;
    
    alpha
}

pub fn alpha_beta_min(alpha: f64, beta: f64, depth_left: i8, game: &mut game::GameInfo, pv: &mut Vec<move_gen::Move>, start_time: &Instant, time_limit: &Duration, ply: i8) -> f64 {
    
    let mut movements = move_gen::move_gen(game);

    if depth_left == 0 || movements.len() == 0{
        return eval::static_evaluate(game);
    }

    let mut beta = beta;
    let mut alpha = alpha;
    
    let index = (game.hash % game::TRANSPOSITION_TABLE_SIZE) as usize;

    if game.transposition_table[index].zobrist_key == game.hash && game.transposition_table[index].depth >= depth_left{
        match game.transposition_table[index].flag{
            game::Flag::Exact => return game.transposition_table[index].value,
            game::Flag::Lowerbound => alpha = alpha.max(game.transposition_table[index].value),
            game::Flag::Upperbound => beta = beta.min(game.transposition_table[index].value),
        }
        if alpha >= beta {
            return game.transposition_table[index].value;
        }
    }
    
    let mut value = 100.0;
    let mut new_pv: Vec<move_gen::Move> = Vec::new();
    eval::order_moves(&mut movements, &pv, game, ply);
    let mut first = true;

    for mut movement in movements {

        if start_time.elapsed() >= *time_limit {
            return 100.0;
        }

        make_move::make_move(game, &mut movement);
        
        let mut score;
        if first{
            score = alpha_beta_max(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1);
            first = false;
        } else {
            score = alpha_beta_max(alpha, alpha + 1.0, depth_left - 1, game, &mut new_pv,start_time, time_limit, ply+1);
            if score > alpha && score < beta {
                score = alpha_beta_max(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1);
            }
        }
        
        unmake::unmake_move(game, movement);

        if score <= alpha {
            if movement.destiny_piece == piece::Piece::Empty{

                let side = if game.turn == game::Color::White {0} else {1};
                game.historic_heuristic[side][movement.origin as usize][movement.destiny as usize] +=( depth_left * depth_left) as usize;
                
                if game.killer_move[ply as usize][0] != movement{
                    game.killer_move[ply as usize][1] = game.killer_move[ply as usize][0];
                    game.killer_move[ply as usize][0] = movement;
                }
            }
            return alpha; 
        }

        if score < beta {
            beta = score;
            pv.clear();
            pv.push(movement);
            pv.append(&mut new_pv);
            value = score;
        }
    }
    
    if beta <= alpha {
        game.transposition_table[index].flag = game::Flag::Upperbound
    } else if beta >= beta {
        game.transposition_table[index].flag = game::Flag::Lowerbound
    } else {
        game.transposition_table[index].flag = game::Flag::Exact
    }

    game.transposition_table[index].zobrist_key = game.hash;
    game.transposition_table[index].depth = depth_left;
    game.transposition_table[index].value = value;
    
    beta
}


pub fn iterative_deepening_time_limit(game: &mut game::GameInfo, max_depth: i8, time_limit: Duration) -> Option<move_gen::Move> {
    let mut best_move: Option<move_gen::Move> = None;
    let start_time = Instant::now();
    let mut pv: Vec<move_gen::Move> = Vec::new();

    for depth in 1..=max_depth {
        let alpha = -100.0;
        let beta = 100.0;
        let score;

        if game.turn == game::Color::White {
            score = alpha_beta_max(alpha, beta, depth, game, &mut pv, &start_time, &time_limit, 1);
        } else {
            score = alpha_beta_min(alpha, beta, depth, game, &mut pv, &start_time, &time_limit, 1);
        }

        if pv.len() > 0 {
            best_move = Some(pv[0]);
        }
        if score == -1.0 || score == 1.0 {
            break;
        }

        if start_time.elapsed() >= time_limit {
            break;
        }
        println!("depth: {} time: {}",depth, start_time.elapsed().as_millis());
    }

    best_move
}

pub fn iterative_deepening_time_limit_net(
    game: &mut game::GameInfo,
    max_depth: i8,
    time_limit: Duration,
    net: &model::Net,
    ) -> Option<move_gen::Move> {
    let mut best_move: Option<move_gen::Move> = None;
    let start_time = Instant::now();
    let mut pv: Vec<move_gen::Move> = Vec::new();
        
    for depth in 1..=max_depth {
        let alpha = -100.0;
        let beta = 100.0;
        let score;
    
        if game.turn == game::Color::White {
            score = alpha_beta_max_net(alpha, beta, depth, game, &mut pv, &start_time, time_limit, net);
        } else {
            score = alpha_beta_min_net(alpha, beta, depth, game, &mut pv, &start_time, time_limit, net);
        }
    
        if pv.len() > 0 {
            best_move = Some(pv[0]);
        }
        if score == -1.0 || score == 1.0 {
            break;
        }
    
        if start_time.elapsed() >= time_limit {
            break;
        }
    }
    
    best_move
    
}