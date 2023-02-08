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

    let re = Regex::new("([RBQKPN])?([a-h])?([1-8])?([x])?([a-h])([1-8])([=]?)([QNRB]?)([+#]?)").unwrap().captures(notation).unwrap();

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
        None => 0,
    };

    let group3 = match re.get(3){
        Some(x) => x.as_str().parse::<u32>().unwrap(),
        None => 0,
    };

    let capture:bool = match re.get(4){
        Some(x) => true,
        None => false,
    
    };

    let group5 = match re.get(5){
        Some(x) => fen_reader::letter_to_column(x.as_str().chars().nth(0).unwrap()),
        None => 0,
    };

    let group6 = match re.get(6){
        Some(x) => x.as_str().parse::<u32>().unwrap(),
        None => 0,
    };

    let promotion:Option<PieceType> = match re.get(7){
        Some(x) =>match re.get(8){
                Some(x) => match x.as_str(){
                    "Q" => Some(PieceType::Queen),
                    "R" => Some(PieceType::Rook),
                    "B" => Some(PieceType::Bishop),
                    "N" => Some(PieceType::Knight),
                    _ => panic!(""),
                },
                None => panic!(""),
            },
        None => None,
    };

    let mut origin = 0;
    let mut destiny = 0;

    if group3 != 0{
        origin = fen_reader::row_column_to_index(&group3, &group2)
    }

    


    move_gen::Move { origin: origin, destiny: destiny, destiny_piece: piece::Piece::Empty, promotion: promotion }
}

