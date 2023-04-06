use std::{env, io::BufRead, fs};
use regex;
use chess::{model, zobrist_hashing::HASH, training_parser, fen_reader, alpha_beta_search::alpha_beta_min};
use tch;
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    unsafe {
        HASH.randomize();
    }
    /* 
    tch::set_num_threads(4);
    println!("{}",tch::get_num_threads());
    model::train()
    */
    
    training_parser::get_training_fen("./historical_games")
    
}
