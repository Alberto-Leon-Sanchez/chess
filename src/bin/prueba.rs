use std::{env, io::{BufRead, Write}, fs};
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
    model::train();
    */

    let file = fs::File::open("/home/castorcabron/proyectos/chess/suite2.txt").unwrap();
    let file = std::io::BufReader::new(file);
    let re = regex::Regex::new(r"Score:(\d+)").unwrap();

    let mut output = fs::File::create("/home/castorcabron/proyectos/chess/suite2_results.txt").unwrap();


    for line in file.lines() {
        let line = line.unwrap();
        let captures = re.captures(&line).unwrap();
        
        let score = captures.get(1).unwrap().as_str().parse::<i64>().unwrap();
        output.write_all(format!("{}\n", score).as_bytes());
    }



}
