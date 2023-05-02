use std::time::Duration;
use std::time::Instant;

use rayon::prelude::IntoParallelIterator;
use rayon::prelude::ParallelIterator;

use crate::eval;
use crate::game;
use crate::make_move;
use crate::model;
use crate::move_gen;
use crate::piece;
use crate::unmake;

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
    let tt = game.transposition_table.lock().unwrap();

    if tt[index].zobrist_key == game.hash && tt[index].depth >= depth_left{
        match tt[index].flag{
            game::Flag::Exact => return tt[index].value,
            game::Flag::Lowerbound => alpha = alpha.max(tt[index].value),
            game::Flag::Upperbound => beta = beta.min(tt[index].value),
        }
        if alpha >= beta {
            return tt[index].value;
        }
    }
    drop(tt);
    
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
    let mut tt = game.transposition_table.lock().unwrap();

    tt[index].zobrist_key = game.hash;
    tt[index].flag = flag;
    tt[index].depth = depth_left;
    tt[index].value = alpha;
    drop(tt);

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
    let tt = game.transposition_table.lock().unwrap();
    if tt[index].zobrist_key == game.hash && tt[index].depth >= depth_left{
        match tt[index].flag{
            game::Flag::Exact => return tt[index].value,
            game::Flag::Lowerbound => alpha = alpha.max(tt[index].value),
            game::Flag::Upperbound => beta = beta.min(tt[index].value),
        }
        if alpha >= beta {
            return tt[index].value;
        }
    }
    drop(tt);
    
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
    let mut tt = game.transposition_table.lock().unwrap();

    tt[index].zobrist_key = game.hash;
    tt[index].flag = flag;
    tt[index].depth = depth_left;
    tt[index].value = beta;
    drop(tt);

    beta
    
}
   

pub fn alpha_beta_max(alpha: f64, beta: f64, depth_left: i8, game: &mut game::GameInfo, pv: &mut Vec<move_gen::Move>,start_time: &Instant, time_limit: &Duration, ply: i8, max_depth: i8) -> f64 {

    let mut movements = move_gen::move_gen(game);
    
    if depth_left == 0 || movements.len() == 0 {
        return eval::static_evaluate(game);
    }

    let mut alpha = alpha;
    let mut beta = beta;
    
    let index = (game.hash % game::TRANSPOSITION_TABLE_SIZE) as usize;
    let tt = game.transposition_table.lock().unwrap();
    if tt[index].zobrist_key == game.hash && tt[index].depth >= depth_left{
        match tt[index].flag{
            game::Flag::Exact => return tt[index].value,
            game::Flag::Lowerbound => alpha = alpha.max(tt[index].value),
            game::Flag::Upperbound => beta = beta.min(tt[index].value),
        }
        if alpha >= beta {
            return tt[index].value;
        }
    }
    drop(tt);

    let mut new_pv: Vec<move_gen::Move> = Vec::new();
    let mut first = false;
    let mut value = -100.0;

    eval::order_moves(&mut movements, &pv, game, ply);
    if depth_left == max_depth{

        let mut pv_move = movements.remove(0);
        let mut new_pv: Vec<move_gen::Move> = Vec::new();
        make_move::make_move(game, &mut pv_move);
        let pv_score = alpha_beta_min(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply + 1, max_depth);
        unmake::unmake_move(game, pv_move);

        if pv_score >= beta {
            if pv_move.destiny_piece == piece::Piece::Empty {
                let side = if game.turn == game::Color::White { 0 } else { 1 };
                game.historic_heuristic.lock().unwrap()[side][pv_move.origin as usize][pv_move.destiny as usize] += (depth_left) as usize;
                let mut killer_move = game.killer_move.lock().unwrap();

                if killer_move[ply as usize][0] != pv_move {
                    killer_move[ply as usize][1] = killer_move[ply as usize][0];
                    killer_move[ply as usize][0] = pv_move;
                }
            }
            return beta;
        }

        if pv_score > alpha {
            alpha = pv_score;
            pv.clear();
            pv.push(pv_move);
            pv.append(&mut new_pv);
            value = pv_score;
        }
        
        let result: Vec<_> = movements
            .into_par_iter()
            .map(|mut movement| {
                let mut game_clone = game.clone();
                let mut new_pv: Vec<move_gen::Move> = Vec::new();

                make_move::make_move(&mut game_clone, &mut movement);

                let mut score = alpha_beta_min(beta - 1.0, beta, depth_left - 1, &mut game_clone, &mut new_pv, start_time, time_limit, ply + 1, max_depth);
                if score < beta && score > beta - 1.0 {
                    score = alpha_beta_min(alpha, beta, depth_left - 1, &mut game_clone, &mut new_pv, start_time, time_limit, ply + 1, max_depth);
                }

                unmake::unmake_move(&mut game_clone, movement);

                (score, movement, new_pv)
            })
            .collect();

        for (score, movement, mut new_pv) in result {
            if score >= beta {
                if movement.destiny_piece == piece::Piece::Empty {
                    let side = if game.turn == game::Color::White { 0 } else { 1 };
                    game.historic_heuristic.lock().unwrap()[side][movement.origin as usize][movement.destiny as usize] += (depth_left) as usize;
                    let mut killer_move = game.killer_move.lock().unwrap();
                    if killer_move[ply as usize][0] != movement {
                        killer_move[ply as usize][1] = killer_move[ply as usize][0];
                        killer_move[ply as usize][0] = movement;
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

    }else{
        for mut movement in movements {

            if start_time.elapsed() >= *time_limit {
                return -100.0;
            }

            make_move::make_move(game, &mut movement);
            
            let mut score;
            if first{
                score = alpha_beta_min(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1, max_depth);
                first = false;
            }else{
                score = alpha_beta_min(beta - 1.0, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1, max_depth);
                if score > alpha && score < beta {
                    score = alpha_beta_min(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1, max_depth);
                }
            }
            unmake::unmake_move(game, movement);

            if score >= beta {
                if movement.destiny_piece == piece::Piece::Empty{

                    let side = if game.turn == game::Color::White {0} else {1};
                    game.historic_heuristic.lock().unwrap()[side][movement.origin as usize][movement.destiny as usize] +=( depth_left) as usize;
                    let mut killer_move = game.killer_move.lock().unwrap();
                    if killer_move[ply as usize][0] != movement{
                        killer_move[ply as usize][1] = killer_move[ply as usize][0];
                        killer_move[ply as usize][0] = movement;
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
    }   
    
    let mut tt = game.transposition_table.lock().unwrap();
    if depth_left >= tt[index].depth{
        if alpha <= alpha {
            tt[index].flag = game::Flag::Upperbound;
        } else if alpha >= beta {
            tt[index].flag = game::Flag::Lowerbound;
        } else {
            tt[index].flag = game::Flag::Exact;
        }
        
        tt[index].zobrist_key = game.hash;
        tt[index].depth = depth_left;
        tt[index].value = value;
    }
    drop(tt);
    
    alpha
}

pub fn alpha_beta_min(alpha: f64, beta: f64, depth_left: i8, game: &mut game::GameInfo, pv: &mut Vec<move_gen::Move>, start_time: &Instant, time_limit: &Duration, ply: i8, max_depth: i8) -> f64 {
    
    let mut movements = move_gen::move_gen(game);

    if depth_left == 0 || movements.len() == 0{
        return eval::static_evaluate(game);
    }

    let mut beta = beta;
    let mut alpha = alpha;
    
    let index = (game.hash % game::TRANSPOSITION_TABLE_SIZE) as usize;
    let tt = game.transposition_table.lock().unwrap();
    if tt[index].zobrist_key == game.hash && tt[index].depth >= depth_left{
        match tt[index].flag{
            game::Flag::Exact => return tt[index].value,
            game::Flag::Lowerbound => alpha = alpha.max(tt[index].value),
            game::Flag::Upperbound => beta = beta.min(tt[index].value),
        }
        if alpha >= beta {
            return tt[index].value;
        }
    }
    drop(tt);
    
    let mut value = 100.0;
    let mut new_pv: Vec<move_gen::Move> = Vec::new();
    eval::order_moves(&mut movements, &pv, game, ply);
    let mut first = true;

    if depth_left == max_depth{

        let mut pv_move = movements.remove(0);
        let mut new_pv: Vec<move_gen::Move> = Vec::new();
        make_move::make_move(game, &mut pv_move);
        let pv_score = alpha_beta_max(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply + 1, max_depth);
        unmake::unmake_move(game, pv_move);

        if pv_score <= alpha {
            if pv_move.destiny_piece == piece::Piece::Empty {
                let side = if game.turn == game::Color::White { 0 } else { 1 };
                game.historic_heuristic.lock().unwrap()[side][pv_move.origin as usize][pv_move.destiny as usize] += (depth_left) as usize;
                let mut killer_move = game.killer_move.lock().unwrap();

                if killer_move[ply as usize][0] != pv_move {
                    killer_move[ply as usize][1] = killer_move[ply as usize][0];
                    killer_move[ply as usize][0] = pv_move;
                }
            }
            return alpha;
        }

        if pv_score < beta {
            beta = pv_score;
            pv.clear();
            pv.push(pv_move);
            pv.append(&mut new_pv);
            value = pv_score;
        }

        let result: Vec<_> = movements
        .into_par_iter()
        .map(|mut movement| {
            let mut game_clone = game.clone();
            let mut new_pv: Vec<move_gen::Move> = Vec::new();

            make_move::make_move(&mut game_clone, &mut movement);
            
            let mut score = alpha_beta_max(alpha, alpha + 1.0, depth_left - 1, &mut game_clone, &mut new_pv, start_time, time_limit, ply + 1, max_depth);
            if score > alpha && score < beta {
                score = alpha_beta_max(alpha, beta, depth_left - 1, &mut game_clone, &mut new_pv, start_time, time_limit, ply + 1, max_depth);
            }

            unmake::unmake_move(&mut game_clone, movement);

            (score, movement, new_pv)
        })
        .collect();

        for (score, movement, mut new_pv) in result {
            if score <= alpha {
                if movement.destiny_piece == piece::Piece::Empty {
                    let side = if game.turn == game::Color::White { 0 } else { 1 };
                    game.historic_heuristic.lock().unwrap()[side][movement.origin as usize][movement.destiny as usize] += (depth_left) as usize;
                    let mut killer_move = game.killer_move.lock().unwrap();

                    if killer_move[ply as usize][0] != movement {
                        killer_move[ply as usize][1] = killer_move[ply as usize][0];
                        killer_move[ply as usize][0] = movement;
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
    }else{

        for mut movement in movements {

            if start_time.elapsed() >= *time_limit {
                return 100.0;
            }
    
            make_move::make_move(game, &mut movement);
            
            let mut score;
            if first{
                score = alpha_beta_max(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1, max_depth);
                first = false;
            } else {
                score = alpha_beta_max(alpha, alpha + 1.0, depth_left - 1, game, &mut new_pv,start_time, time_limit, ply+1, max_depth);
                if score > alpha && score < beta {
                    score = alpha_beta_max(alpha, beta, depth_left - 1, game, &mut new_pv, start_time, time_limit, ply+1, max_depth);
                }
            }
            
            unmake::unmake_move(game, movement);
    
            if score <= alpha {
                if movement.destiny_piece == piece::Piece::Empty{
    
                    let side = if game.turn == game::Color::White {0} else {1};
                    game.historic_heuristic.lock().unwrap()[side][movement.origin as usize][movement.destiny as usize] +=( depth_left) as usize;
                    let mut killer_move = game.killer_move.lock().unwrap();
    
                    if killer_move[ply as usize][0] != movement{
                        killer_move[ply as usize][1] = killer_move[ply as usize][0];
                        killer_move[ply as usize][0] = movement;
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
    }

    let mut tt = game.transposition_table.lock().unwrap();
    if depth_left >= tt[index].depth{
        if beta <= alpha {
            tt[index].flag = game::Flag::Upperbound
        } else if beta >= beta {
            tt[index].flag = game::Flag::Lowerbound
        } else {
            tt[index].flag = game::Flag::Exact
        }

        tt[index].zobrist_key = game.hash;
        tt[index].depth = depth_left;
        tt[index].value = value;
    }
    drop(tt);

    beta
}


pub fn iterative_deepening_time_limit(game: &mut game::GameInfo, max_depth: i8, time_limit: Duration) -> (Option<(move_gen::Move)>,f64) {
    let mut best_move: Option<move_gen::Move> = None;
    let mut pv: Vec<move_gen::Move> = Vec::new();
    let mut score = 0.0;
    let start_time = Instant::now();

    for depth in 1..=max_depth {
        let alpha = -100.0;
        let beta = 100.0;

        if game.turn == game::Color::White {
            score = alpha_beta_max(alpha, beta, depth, game, &mut pv, &start_time, &time_limit, 1, depth);
        } else {
            score = alpha_beta_min(alpha, beta, depth, game, &mut pv, &start_time, &time_limit, 1, depth);
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

    (best_move,score)
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