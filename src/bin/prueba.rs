use std::{hash, env, fs::{self, File}, io::BufReader};

use chess::{
    fen_reader, perft::perft, zobrist_hashing::HASH, alpha_beta_search, model, training_parser, eval, move_gen::{move_gen, self}, make_move, unmake, suite
};
use tch::nn::{self, Module};

fn main(){
    
    env::set_var("RUST_BACKTRACE", "1");
    unsafe{
        HASH.randomize();
    }
    
    model::train()

}
