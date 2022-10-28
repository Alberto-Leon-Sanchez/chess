use std::hash;

use chess::{
    fen_reader, perft::perft, zobrist_hashing::HASH, alpha_beta_search,
};

fn main(){
    
    unsafe{
        HASH.randomize();
    }


    let mut game = fen_reader::read_fen("6k1/pp2brp1/4Q2p/8/8/1PnrB3/P4PPP/R5K1 b - - 1 21");
    
    let best_move = alpha_beta_search::get_best(&mut game, 6);

    println!("{},{}",best_move.origin,best_move.destiny);
    

}
