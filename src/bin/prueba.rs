use std::{env, io::BufRead, fs, io::{BufReader, Write}, time::Duration, process};
use regex;
use chess::{model, zobrist_hashing::HASH, training_parser, fen_reader, alpha_beta_search::{alpha_beta_min, self, iterative_deepening_time_limit}, uci, suite, game};
use tch;
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::var("RUST_MIN_STACK").unwrap_or_else(|_| "167772160".to_string());
    
    unsafe {
        HASH.randomize();
    }
    /* 

    let mut suites = suite::get_suites();
    println!("{}",suite::test_model_net(None, &mut suites, 0));
     tch::set_num_threads(4);
    println!("{}",tch::get_num_threads());
    model::train();
    */

    rayon::ThreadPoolBuilder::new()
    .stack_size(8388608)
    .num_threads(12)
    .build_global()
    .unwrap();

    //println!("{}", suite::test_engine("stockfish", Duration::from_millis(100)));
    let mut game = game::GameInfo::new();
    //println!("{:?}",iterative_deepening_time_limit(&mut game, 100, Duration::from_millis(10000)));
    println!("{}", suite::test_model_net(None, &mut suite::get_suites(), 0, Duration::from_millis(100)));
    println!("{}", suite::test_model_net(None, &mut suite::get_suites(), 0, Duration::from_millis(500)));
    println!("{}", suite::test_model_net(None, &mut suite::get_suites(), 0, Duration::from_millis(1000)));


    


}
