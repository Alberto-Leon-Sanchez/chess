use crate::game;
use crate::game::GameInfo;
use crate::move_gen;
use crate::fen_reader;
use crate::move_gen::DIAGONAL_SLIDING;
use crate::move_gen::LATERAL_SLIDING;
use crate::piece;
use crate::piece::Piece;
use crate::piece::PieceType;
use regex::Regex;
use serde::__private::de;

pub fn get_move(notation: &str, game: &mut game::GameInfo) -> move_gen::Move{

    let turn = &game.turn;
    
    if notation == "O-O"{
        
        let mut origin = 25;
        let mut destiny = 27;

        if game.turn == game::Color::Black{
            origin = 95;
            destiny = 97;
        }
      
        return move_gen::Move{
            origin: origin,
            destiny: destiny,
            destiny_piece: piece::Piece::Empty,
            promotion: None,
        }
    }
    if notation == "O-O-O"{
        
        let mut origin = 25;
        let mut destiny = 23;

        if game.turn == game::Color::Black{
            origin = 95;
            destiny = 93;    
        }
        
        return move_gen::Move{
            origin: origin,
            destiny: destiny,
            destiny_piece: piece::Piece::Empty,
            promotion: None,
        }
    }

    let re = Regex::new("([RBQKPN])?([a-h])?([1-8])?([x])?([a-h])?([1-8])?([=]?)([QNRB]?)([+#]?)").unwrap().captures(notation).unwrap();

    let piece:piece::Piece = match re.get(1){
        Some(x) => match x.as_str(){
            "R" => {if turn == &game::Color::White {piece::Piece::White(piece::PieceType::Rook)} else {piece::Piece::Black(piece::PieceType::Rook)}},
            "B" => {if turn == &game::Color::White {piece::Piece::White(piece::PieceType::Bishop)} else {piece::Piece::Black(piece::PieceType::Bishop)}},
            "Q" => {if turn == &game::Color::White {piece::Piece::White(piece::PieceType::Queen)} else {piece::Piece::Black(piece::PieceType::Queen)}},
            "K" => {if turn == &game::Color::White {piece::Piece::White(piece::PieceType::King)} else {piece::Piece::Black(piece::PieceType::King)}},
            "N" => {if turn == &game::Color::White {piece::Piece::White(piece::PieceType::Knight)} else {piece::Piece::Black(piece::PieceType::Knight)}},            
            _ => panic!(""),
        },
        None => {if turn == &game::Color::White {piece::Piece::White(piece::PieceType::Pawn)} else {piece::Piece::Black(piece::PieceType::Pawn)}},
    };

    let group2 = match re.get(2){
        Some(x) => fen_reader::letter_to_column(x.as_str().chars().nth(0).unwrap()),
        None => 200,
    };

    let group3 = match re.get(3){
        Some(x) => x.as_str().parse::<u32>().unwrap() - 1,
        None => 200,
    };

    let capture:bool = match re.get(4){
        Some(_) => true,
        None => false,
    
    };

    let group5 = match re.get(5){
        Some(x) => fen_reader::letter_to_column(x.as_str().chars().nth(0).unwrap()),
        None => 0,
    };

    let group6 = match re.get(6){
        Some(x) => x.as_str().parse::<u32>().unwrap() - 1,
        None => 0,
    };

    let promotion:Option<PieceType> = match re.get(7){
        Some(_) => match re.get(8){
                Some(x) => match x.as_str(){
                    "Q" => Some(PieceType::Queen),
                    "R" => Some(PieceType::Rook),
                    "B" => Some(PieceType::Bishop),
                    "N" => Some(PieceType::Knight),
                    _ => None,
                },
                None => panic!(""),
            },
        None => None,
    };

    let mut origin;
    let mut destiny = 0;
    let mut destiny_piece = piece::Piece::Empty;

    if capture{
        destiny = fen_reader::row_column_to_index(&group6, &group5);
        destiny_piece = game.board[destiny as usize];

        if piece == piece::Piece::White(piece::PieceType::Pawn){
            let diference;

            if group2 > group5 {
                diference = 9;
            }else{
                diference = 11;
            }

            origin = ((destiny) as i32 - diference) as u32;

        }else if piece == piece::Piece::Black(piece::PieceType::Pawn){
            
            let diference;
            
            if group2 > group5 {
                diference = -11;
            }else{
                diference = -9;
            }

            origin = ((destiny) as i32 - diference) as u32;

        }else{

            let origins = get_origin(destiny, game, piece);

            if origins.len() > 1{
                if group2 != 200{
                    origin = get_piece_in_column(origins, group2);
                }else{
                    origin = get_piece_in_rank(origins, group3);
                }
            }else{
                origin = origins[0] as u32;
            }

        }

    }else{

        if group3 != 200 && group2 != 200{
            destiny = fen_reader::row_column_to_index(&group3, &group2)
        }

        if piece == piece::Piece::White(piece::PieceType::Pawn){

            if game.board[destiny - 10 as usize] == piece::Piece::White(piece::PieceType::Pawn){
                origin = (destiny - 10) as u32;
            }else{
                origin = (destiny - 20) as u32;
            }
        }else if piece == piece::Piece::Black(piece::PieceType::Pawn){

            if game.board[destiny + 10 as usize] == piece::Piece::Black(piece::PieceType::Pawn){
                origin = (destiny + 10) as u32;
            }else{
                origin = (destiny + 20) as u32;
            }
        }else{

            let origins;
            if destiny == 0{
                destiny = fen_reader::row_column_to_index(&group6, &group5);
                origins = get_origin(destiny, game, piece);

                if group2 != 200{
                    origin = get_piece_in_column(origins, group2);
                }else{
                    origin = get_piece_in_rank(origins,group3);
                }
                
            }else{
                origins = get_origin(destiny, game, piece);
                origin = origins[0] as u32;
            }


        }
    
        
    }


    move_gen::Move { origin: origin as i8, destiny: destiny as i8, destiny_piece: destiny_piece, promotion: promotion }
}

fn get_origin(destiny:usize, game: &mut game::GameInfo, piece: piece::Piece) -> Vec<usize>{
    let moves  = move_gen::move_gen(game);
    let mut origin = vec![];
    
    for movement in moves{
        if movement.destiny == destiny as i8 && game.board[movement.origin as usize] == piece{
            origin.push(movement.origin as usize);
        }
    }

    origin
}

fn get_piece_in_column(origins: Vec<usize>,column: u32) -> u32{
    for origin in &origins{
        if (origin % 10 - 1) == column as usize{
            return *origin as u32;
        }
    }

    panic!("{:?},{}",origins,column);
}

fn get_piece_in_rank(origins: Vec<usize>,rank: u32) -> u32{
    for origin in &origins{
        if (origin / 10 - 2) == rank as usize{
            return *origin as u32;
        }
    }

    panic!("{:?},{}",origins,rank);
}

