use crate::game;
use crate::move_gen;
use crate::make_move;
use crate::unmake;

pub fn alpha_beta_max(alpha:i8,beta:i8,depth:i8,game:&mut game::GameInfo) -> i8 {
    if depth == 0{
        //todo
    }
    let mut alpha = alpha;
    let mut v = i8::MIN;
    let moves = move_gen::move_gen(game);

    for mut movement in moves {
        
        make_move::make_move(game, &mut movement);
        v = std::cmp::max(v,alpha_beta_min(alpha,beta,depth-1,game));
        unmake::unmake_move(game, movement);

        if v >= beta {
            return v;
        }

        alpha = std::cmp::max(alpha,v);
    }
    return v;
}

pub fn alpha_beta_min(alpha:i8,beta:i8,depth:i8,game:&mut game::GameInfo) -> i8 {

    if depth == 0{
        //todo
    }

    let mut beta = beta;
    let mut v = i8::MAX;
    let moves = move_gen::move_gen(game);

    for mut movement in moves {
        
        make_move::make_move(game, &mut movement);
        v = std::cmp::min(v,alpha_beta_max(alpha,beta,depth-1,game));
        unmake::unmake_move(game, movement);

        if v <= alpha {
            return v;
        }

        beta = std::cmp::min(beta,v);
    }
    return v;

}
