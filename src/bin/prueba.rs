use std::env;

use chess::{model, zobrist_hashing::HASH, training_parser};
use tch;
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    unsafe {
        HASH.randomize();
    }
    tch::set_num_threads(4);
    println!("{}",tch::get_num_threads());
    model::train()

}
