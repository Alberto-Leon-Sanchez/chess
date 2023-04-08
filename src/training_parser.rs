use std::fs::{File, OpenOptions, read_dir};
use std::io::{BufRead, BufReader, Write};

use rand::Rng;

use crate::{fen_reader, fen_writer, game, make_move, notation, move_gen, unmake, eval};

pub fn get_training_fen(path:&str) -> (){
    
    let path = read_dir(path)
        .unwrap()
        .map(|x| x.unwrap().path().to_str().unwrap().to_string())
        .collect::<Vec<String>>();

    parse_to_fen(path);
}

fn parse_to_fen(files: Vec<String>) {
    
    let mut writer = OpenOptions::new()
            .append(true)
            .open("./training_fen.txt")
            .unwrap();

    let movement_number = regex::Regex::new(r"([0-9]*\.)").unwrap();
    let movement = regex::Regex::new(
        r"(([RBQKPN]?[1-8]?[a-h]?[x]?[a-h][1-8][=]?[RBQKPN]?)?((O-O-O)?(O-O)?))",
    )
    .unwrap();
    
    let mut rng = rand::thread_rng();
    let mut last_move:move_gen::Move = move_gen::Move::new();

    for path in files {
        
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let mut checkpoint: i64 = 0;

        for line in reader.lines().skip(checkpoint.try_into().unwrap()) {
            let mut game =
                fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

            if line.as_ref().unwrap().starts_with("1.") {
                let a = movement_number
                    .split(line.as_ref().unwrap())
                    .collect::<Vec<&str>>();
                let max_turn = rng.gen_range(3..25);

                for (index, i) in a.iter().enumerate() {

                    if index > max_turn {
                        
                        if rng.gen_bool(0.5){
                            unmake::unmake_move(&mut game, last_move);
                        }
                        
                        if game.turn == game::Color::White{
                            if eval::check(&mut game, game::Color::White){
                                game.turn = game::Color::Black;
                            }
                        }else{
                            if eval::check(&mut game, game::Color::Black){
                                game.turn = game::Color::White;
                            }
                        }

                        writer
                            .write(fen_writer::write_fen(&game).as_bytes())
                            .unwrap();
                        writer.write(b"\n");
                        break;
                    }

                    for b in movement.captures_iter(i) {
                        if b.get(0).unwrap().as_str() != "" {
                            let mut m = notation::get_move(b.get(0).unwrap().as_str(), &mut game);
                            make_move::make_move(&mut game, &mut m);
                            last_move = m;
                        }
                    }
                }
            }
        }
    }
}
