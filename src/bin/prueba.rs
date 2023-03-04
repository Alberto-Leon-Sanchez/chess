use std::env;

use chess::{model, zobrist_hashing::HASH};

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    unsafe {
        HASH.randomize();
    }

    model::train()
}
