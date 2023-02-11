use std::fs::File;
use std::io::{BufReader, BufRead};

use crate::{game, fen_reader, notation, make_move};

pub fn test() -> Vec<game::GameInfo>{

    const path:&str = "/home/castor_cabron/proyectos/chess/ficsgamesdb_2022_chess2000_nomovetimes_275461.pgn";
    let mut games: Vec<game::GameInfo> = Vec::new();

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let movement_number = regex::Regex::new(r"([0-9]*\.)").unwrap();
    let movement = regex::Regex::new(r"(([RBQKPN]?[a-h]?[x]?[a-h][1-8][=]?[RBQKPN]?)?((O-O)?(O-O-O)?))").unwrap();

    for line in reader.lines() {
        
        let mut game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

        if line.as_ref().unwrap().starts_with("1."){
            let a = movement_number.split(line.as_ref().unwrap()).collect::<Vec<&str>>();

            for (index,i) in a.iter().enumerate(){
                
                if index > 12{
                    game.print_board();
                    games.push(game);
                    println!("{}",games.len());
                    break;
                }
                
                for b in movement.captures_iter(i){
                    if b.get(0).unwrap().as_str() != ""{
                        
                       let mut m = notation::get_move(b.get(0).unwrap().as_str(), &mut game);
                       println!("{},{:?}",b.get(0).unwrap().as_str(),m);

                       make_move::make_move(&mut game, &mut m);

                    }
                }
            
            
            }
        
        }
    }

    games
}