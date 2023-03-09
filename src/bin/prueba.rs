use std::env;

use chess::{model, zobrist_hashing::HASH, training_parser};

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    unsafe {
        HASH.randomize();
    }

    model::train()

}
