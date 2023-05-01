use std::{env, io::BufRead, fs, io::{BufReader, Write}, time::Duration, process};
use regex;
use chess::{model, zobrist_hashing::HASH, training_parser, fen_reader, alpha_beta_search::{alpha_beta_min, self}, uci, suite, game};
use tch;
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    env::var("RUST_MIN_STACK").unwrap_or_else(|_| "167772160".to_string());
    
    unsafe {
        HASH.randomize();
    }
    /* 
    tch::set_num_threads(4);
    println!("{}",tch::get_num_threads());
    model::train();
    

    let mut results:Vec<uci::WinSide> = vec![];

    println!("{:?}",uci::play_game("stockfish", 0, chess::game::Color::White, None, Duration::from_millis(2000)));
    let mut suites = suite::get_suites();
    println!("{}",suite::test_model_net(None, &mut suites, 0));
    */

    //let mut game = fen_reader::read_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ");
    rayon::ThreadPoolBuilder::new()
    .stack_size(8388608)
    .num_threads(10)
    .build_global()
    .unwrap();
    let mut game = game::GameInfo::new();
    println!("{:?}",alpha_beta_search::iterative_deepening_time_limit(&mut game, 100, Duration::from_millis(10000)));

    

}
