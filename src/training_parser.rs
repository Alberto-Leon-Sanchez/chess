use std::fs::{File, self, OpenOptions};
use std::io::{BufReader, BufRead, Write};

use rand::Rng;

use crate::{game, fen_reader, notation, make_move, fen_writer};

pub fn test() {

    const path:&str = "/home/castor_cabron/proyectos/chess/ficsgamesdb_2022_chess2000_nomovetimes_275461.pgn";

    let mut writer = OpenOptions::new().append(true).open("/home/castor_cabron/proyectos/chess/training_fen.txt").unwrap();
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let movement_number = regex::Regex::new(r"([0-9]*\.)").unwrap();
    let movement = regex::Regex::new(r"(([RBQKPN]?[1-8]?[a-h]?[x]?[a-h][1-8][=]?[RBQKPN]?)?((O-O-O)?(O-O)?))").unwrap();
    let mut rng = rand::thread_rng();
    let mut checkpoint:i64 = 112185;

    for line in reader.lines().skip(checkpoint.try_into().unwrap()) {
        
        let mut game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        checkpoint += 1;

        if line.as_ref().unwrap().starts_with("1."){
            let mut f_checkpoint = OpenOptions::new().create(true).write(true).truncate(true).open("/home/castor_cabron/proyectos/chess/checkpoint.txt").unwrap();
            f_checkpoint.write_all(checkpoint.to_string().as_bytes()).unwrap();

            let a = movement_number.split(line.as_ref().unwrap()).collect::<Vec<&str>>();
            let max_turn = rng.gen_range(12..18);
            for (index,i) in a.iter().enumerate(){

                if index > max_turn{
                    writer.write(fen_writer::write_fen(&game).as_bytes()).unwrap();
                    writer.write(b"\n");
                    break;
                }
                
                for b in movement.captures_iter(i){
                    if b.get(0).unwrap().as_str() != ""{
                        
                        let mut m = notation::get_move(b.get(0).unwrap().as_str(), &mut game);
                        make_move::make_move(&mut game, &mut m);

                    }
                }
            
            
            }
        
        }
    }

    
}