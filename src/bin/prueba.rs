use std::hash;

use chess::{
    fen_reader, perft::perft, zobrist_hashing::HASH
};

fn main(){
    
    unsafe{
        HASH.randomize();
    }


    let mut game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ");
    let mut nodes = 0;

    nodes = perft(6, &mut game);

    println!("{}",nodes);
    

}
