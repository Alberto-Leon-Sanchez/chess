use std::char::MAX;
use std::f32::MIN;

use crate::game;
use crate::move_gen;
use crate::make_move;
use crate::piece;
use crate::unmake;
use crate::eval;

static mut hash_access:u64 =0;
static mut alpha_cuts:u64 = 0;
static mut beta_cuts:u64 = 0;

pub fn get_best(game:&mut game::GameInfo,depth:i8) -> move_gen::Move{
    let mut best_move = move_gen::Move{origin:0,destiny:0,destiny_piece:piece::Piece::Empty,promotion:None};
    let mut best_score:i16 = -9999;

    let moves = move_gen::move_gen(game);

    for mut mov in moves{
        make_move::make_move(game,&mut mov);
        let score = alpha_beta_max(i16::MIN,i16::MAX,depth-1,game);

        if score > best_score{
            best_score = score;
            best_move = mov;
        }

        unmake::unmake_move(game,mov);
    }

    println!("{},{}",unsafe {
        beta_cuts
    },unsafe {
        alpha_cuts
    });

    best_move
}

pub fn get_best_negamax_alpha_beta(game:&mut game::GameInfo,depth:i8) -> move_gen::Move{
    
    let mut best_move = move_gen::Move{origin:0,destiny:0,destiny_piece:piece::Piece::Empty,promotion:None};
    let mut best_score:i16 = -9999;

    let moves = move_gen::move_gen(game);

    for mut mov in moves{
        make_move::make_move(game,&mut mov);
        let score = negamax_alpha_beta(game,i16::MIN+1,i16::MAX-1,depth-1);

        if score > best_score{
            best_score = score;
            best_move = mov;
        }

        unmake::unmake_move(game,mov);
    }

    best_move
}

pub fn get_best_with_memory(game:&game::GameInfo,depth:i8) -> move_gen::Move{
    let mut best_move = move_gen::Move{origin:0,destiny:0,destiny_piece:piece::Piece::Empty,promotion:None};
    let mut best_score:i16 = -9999;
    let mut game = game.clone();

    let moves = move_gen::move_gen(&mut game);


    for mut mov in moves{
        make_move::make_move(&mut game,&mut mov);
        let score = alpha_beta_max_with_memory(i16::MIN,i16::MAX,depth-1,&mut game);
        
        if score > best_score{
            best_score = score;
            best_move = mov;
        }

        unmake::unmake_move(&mut game,mov);
    }
    println!("{}",unsafe {hash_access});
    best_move
}

pub fn get_best_iterative_deepening(game:&game::GameInfo,depth:i8) -> move_gen::Move{
    
    let mut best_move = move_gen::Move{origin:0,destiny:0,destiny_piece:piece::Piece::Empty,promotion:None};
    let mut best_score:i16 = -9999;
    let mut game = game.clone();

    let mut moves = move_gen::move_gen(&mut game);
    let mut scores = vec![0;moves.len()];

    for actual_depth in 1..=depth{
        for (i,mov) in moves.iter().enumerate(){
            make_move::make_move(&mut game,&mut mov.clone());
            let score = alpha_beta_max_with_memory(i16::MIN,i16::MAX,actual_depth-1,&mut game);
            
            if score > best_score{
                best_score = score;
                best_move = *mov;
            }

            scores[i] = score;

            unmake::unmake_move(&mut game,*mov);
        }
        
        order_moves(&mut moves,&scores);
        
    }


    println!("{}",unsafe {hash_access});

    best_move
}

fn order_moves(moves:&mut Vec<move_gen::Move>,scores:&Vec<i16>) {
    for i in 0..moves.len()-1{
        for j in i+1..moves.len(){
            if scores[i] < scores[j]{
                moves.swap(i,j);
            }
        }
    }
}

fn negamax_alpha_beta(game:&mut game::GameInfo,alpha:i16,beta:i16,depth:i8) -> i16{
    if depth == 0{
        return eval::eval(game);
    }
    let mut alpha = alpha;

    let moves = move_gen::move_gen(game);

    for mut mov in moves{
        make_move::make_move(game,&mut mov);
        let score = -negamax_alpha_beta(game,-beta,-alpha,depth-1);
        unmake::unmake_move(game,mov);

        if score >= beta{
            return score;
        }

        if score > alpha{
            alpha = score;
        }
    }
    alpha
}

fn negamax_alpha_beta_with_memory(game:&mut game::GameInfo,alpha:i16,beta:i16,depth:i8) -> i16{
    
    let mut alpha = alpha;
    let mut beta = beta;
    let index:usize = (game.hash & game::TRANSPOSITION_TABLE_SIZE) as usize;

    if game.transposition_table[index].zobrist_key == game.hash && game.transposition_table[index].depth >= depth{

        unsafe{hash_access+=1}

        match game.transposition_table[index].flag{
            game::Flag::Exact => return game.transposition_table[index].value,
            game::Flag::Lowerbound => alpha = std::cmp::max(alpha,game.transposition_table[index].value),
            game::Flag::Upperbound => beta = std::cmp::min(beta,game.transposition_table[index].value),
        }
        if alpha >= beta{
            return game.transposition_table[index].value;
        }
    }

    let moves = move_gen::move_gen(game);

    if depth == 0 || moves.len() == 0{
        return eval::eval(game);
    }

    alpha
}

fn alpha_beta_max(alpha:i16,beta:i16,depth:i8,game:&mut game::GameInfo) -> i16 {
    if depth == 0{
        return eval::eval(game);
    }
    let mut alpha = alpha;
    let mut v = i16::MIN;
    let moves = move_gen::move_gen(game);

    for mut movement in moves {
        
        make_move::make_move(game, &mut movement);
        v = std::cmp::max(v,alpha_beta_min(alpha,beta,depth-1,game));
        unmake::unmake_move(game, movement);

        if v >= beta {
            unsafe{beta_cuts+=1}

            return v;
        }

        alpha = std::cmp::max(alpha,v);
    }
    return v;
}

fn alpha_beta_min(alpha:i16,beta:i16,depth:i8,game:&mut game::GameInfo) -> i16 {

    if depth == 0{
        return eval::eval(game);
    }

    let mut beta = beta;
    let mut v = i16::MAX;
    let moves = move_gen::move_gen(game);

    for mut movement in moves {
        
        make_move::make_move(game, &mut movement);
        v = std::cmp::min(v,alpha_beta_max(alpha,beta,depth-1,game));
        unmake::unmake_move(game, movement);

        if v <= alpha {
            unsafe{alpha_cuts+=1}
            return v;
        }

        beta = std::cmp::min(beta,v);
    }
    return v;

}

fn alpha_beta_max_with_memory(alpha:i16,beta:i16,depth:i8,game:&mut game::GameInfo) -> i16 {
    
    let index:usize = (game.hash & game::TRANSPOSITION_TABLE_SIZE) as usize;
    let mut alpha = alpha;
    let mut beta = beta;
    let mut v = i16::MIN;

    if game.transposition_table[index].zobrist_key == game.hash && game.transposition_table[index].depth >= depth{

        unsafe{hash_access+=1}

        match game.transposition_table[index].flag{
            game::Flag::Exact => return game.transposition_table[index].value,
            game::Flag::Lowerbound => alpha = std::cmp::max(alpha,game.transposition_table[index].value),
            game::Flag::Upperbound => beta = std::cmp::min(beta,game.transposition_table[index].value),
        }
        if alpha >= beta{
            return game.transposition_table[index].value;
        }
    }

    if depth == 0{
        return eval::eval(game);
    }

    let moves = move_gen::move_gen(game);

    for mut movement in moves {
        
        make_move::make_move(game, &mut movement);
        v = std::cmp::max(v,alpha_beta_min_with_memory(alpha,beta,depth-1,game));
        unmake::unmake_move(game, movement);

        if v >= beta {
            game.transposition_table[index] = game::Eval{movement:movement,depth:depth,zobrist_key:game.hash,flag:game::Flag::Upperbound,value:v};        
            return v;
        }

        game.transposition_table[index] = game::Eval{movement:movement,depth:depth,zobrist_key:game.hash,flag:game::Flag::Exact,value:v};

        alpha = std::cmp::max(alpha,v);
    }
    return v;
}

fn alpha_beta_min_with_memory(alpha:i16,beta:i16,depth:i8,game:&mut game::GameInfo) -> i16 {

    let index:usize = (game.hash & game::TRANSPOSITION_TABLE_SIZE) as usize;
    let mut alpha = alpha;
    let mut beta = beta;
    let mut v = i16::MAX;

    if game.transposition_table[index].zobrist_key == game.hash && game.transposition_table[index].depth >= depth{
        
        unsafe{hash_access+=1}

        match game.transposition_table[index].flag{
            game::Flag::Exact => return game.transposition_table[index].value,
            game::Flag::Lowerbound => alpha = std::cmp::max(alpha,game.transposition_table[index].value),
            game::Flag::Upperbound => beta = std::cmp::min(beta,game.transposition_table[index].value),
        }
        if alpha >= beta{
            return game.transposition_table[index].value;
        }
    }

    if depth == 0{
        return eval::eval(game);
    }

    let moves = move_gen::move_gen(game);

    for mut movement in moves {
        
        make_move::make_move(game, &mut movement);
        v = std::cmp::min(v,alpha_beta_max_with_memory(alpha,beta,depth-1,game));
        unmake::unmake_move(game, movement);

        if v <= alpha {
            game.transposition_table[index] = game::Eval{movement:movement,depth:depth,zobrist_key:game.hash,flag:game::Flag::Lowerbound,value:v};        
            return v;
        }

        game.transposition_table[index] = game::Eval{movement:movement,depth:depth,zobrist_key:game.hash,flag:game::Flag::Exact,value:v};

        beta = std::cmp::min(beta,v);
    }
    return v;

}