use std::collections::HashMap;

use crate::attack_gen;
use crate::game;
use crate::piece;
use crate::move_gen;

const CENTRAL_CONTROL:[i16;120] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0,
    0, 0, 1, 2, 2, 2, 2, 2, 2, 1, 0, 0,
    0, 0, 1, 2, 3, 3, 3, 3, 2, 1, 0, 0,
    0, 0, 1, 2, 3, 4, 4, 3, 2, 1, 0, 0,
    0, 0, 1, 2, 3, 4, 4, 3, 2, 1, 0, 0,
    0, 0, 1, 2, 3, 3, 3, 3, 2, 1, 0, 0,
    0, 0, 1, 2, 2, 2, 2, 2, 2, 1, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const PAWN:i16 = 100;
const KNIGHT:i16 = 350;
const BISHOP:i16 = 350;
const ROOK:i16 = 525;
const QUEEN:i16 = 1000;

const PAWN_PROTECTED:i16 = 25;
const KNIGHT_PROTECTED:i16 = 15;
const BISHOP_PROTECTED:i16 = 15;
const ROOK_PROTECTED:i16 = 10;
const QUEEN_PROTECTED:i16 = 5;

const PAWN_PROTECION:i16 = 5;
const KNIGHT_PROTECION:i16 = 2;
const BISHOP_PROTECION:i16 = 3;
const ROOK_PROTECION:i16 = 3;
const QUEEN_PROTECION:i16 = 2;
const KING_PROTECTION:i16 = 5;

const PAWN_MOVEMENT:i16 = 2;
const KNIGHT_MOVEMENT:i16 = 3;
const BISHOP_MOVEMENT:i16 = 5;
const ROOK_MOVEMENT:i16 = 5;
const QUEEN_MOVEMENT:i16 = 9;
const KING_MOVEMENT:i16 = 1;


pub fn eval(game:&mut game::GameInfo) -> i16{

    let mut score = 0;
    let mut temp = 0;
    let attacks:[u8;120];

    let turn:i16 = if game.turn == game::Color::White{
        (attacks,_) = attack_gen::attack_gen(game, Some(&game::Color::White));
        1
    }else{
        (attacks,_) = attack_gen::attack_gen(game, Some(&game::Color::Black));
        -1
    };

    let moves = move_gen::move_gen(game);

    let mut mobility:HashMap<i8,i16> = HashMap::new();
    let mut conectivity: HashMap<i8,i16> = HashMap::new();

    
    let piece_list = if game.turn == game::Color::White{
        &game.white_pieces
    }else{
        &game.black_pieces
    };

    temp = material_eval(piece_list);

    //println!("material:{}",temp);
    score += temp;
    //temp = mobility_conectivity_eval(&mut mobility, &mut conectivity, &moves, &game.board, &game.turn);
    //println!("conectivity and mobility:{}",temp);
    //score += temp;
    //temp = center_control_eval(&attacks);
    //println!("center control:{}",temp);
    //score += temp;

    score * turn
}

fn material_eval(piece_list:&piece::PieceList) ->i16{
    let mut score:i16 = 0;
    
    score += (piece_list.pawns.len() as i16) * PAWN;
    score += (piece_list.knights.len() as i16) * KNIGHT;
    score += (piece_list.bishops.len() as i16) * BISHOP;
    score += (piece_list.rooks.len() as i16) * ROOK;
    score += (piece_list.queens.len() as i16) * QUEEN;

    score
}

fn mobility_conectivity_eval(mobility:&mut HashMap<i8,i16>,conectivity:&mut HashMap<i8,i16>, moves:&Vec<move_gen::Move>, board:&[piece::Piece;120],turn:&game::Color) -> i16{

    let mut score:i16 = 0;
    
    for movement in moves{

        if mobility.contains_key(&movement.origin){
            *mobility.get_mut(&movement.origin).unwrap() += 1;
        }else{
            mobility.insert(movement.origin, 1);
        }

        if conectivity.contains_key(&movement.origin){
            match board[movement.destiny as usize]{
                piece::Piece::White(p) => *conectivity.get_mut(&movement.origin).unwrap() *= piece_type_protection(p),
                piece::Piece::Black(p) => *conectivity.get_mut(&movement.origin).unwrap() *= piece_type_protection(p),
                _ => ()
            }
            
        }else{
            let piece_type = match board[movement.origin as usize]{
                piece::Piece::White(p) => Some(p),
                piece::Piece::Black(p) => Some(p),
                _ => None
            }.unwrap();
    
            conectivity.insert(movement.origin, piece_type_protected(piece_type));
        }
    }

    for key in mobility.keys(){
        let piece = match board[*key as usize]{
            piece::Piece::White(p) => Some(p),
            piece::Piece::Black(p) => Some(p),
            _ => None,
        }.unwrap();

        match piece{
            piece::PieceType::Pawn => score += PAWN_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::Knight => score += KNIGHT_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::Bishop => score += BISHOP_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::Rook => score += ROOK_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::Queen => score += QUEEN_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::King => score += KING_MOVEMENT * mobility.get(key).unwrap(),
        }
    }
    for value in conectivity.values(){
        score += *value;
    }

    score

}

fn piece_type_protected(piece:piece::PieceType)-> i16{
    match piece{
        piece::PieceType::Pawn => PAWN_PROTECTED,
        piece::PieceType::Knight => KNIGHT_PROTECTED,
        piece::PieceType::Bishop => BISHOP_PROTECTED,
        piece::PieceType::Rook => ROOK_PROTECTED,
        piece::PieceType::Queen => QUEEN_PROTECTED,
        _ => 0,
    }
}

fn piece_type_protection(piece:piece::PieceType)-> i16{
    match piece{
        piece::PieceType::Pawn => PAWN_PROTECION,
        piece::PieceType::Knight => KNIGHT_PROTECION,
        piece::PieceType::Bishop => BISHOP_PROTECION,
        piece::PieceType::Rook => ROOK_PROTECION,
        piece::PieceType::Queen => QUEEN_PROTECION,
        piece::PieceType::King => KING_PROTECTION,
    }
}


fn center_control_eval(attacks:&[u8;120]) -> i16{
    let mut score:i16 = 0;

    for (index,value) in attacks.iter().enumerate(){
        score += (*value as i16) * CENTRAL_CONTROL[index];
    }

    score
}



fn king_safety_eval() -> i16{
    let mut score:i16 = 0;

    score
}


