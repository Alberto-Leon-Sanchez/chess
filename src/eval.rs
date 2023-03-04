use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use tch::nn;
use tch::nn::Module;

use crate::attack_gen;
use crate::game;
use crate::model;
use crate::model::Net;
use crate::move_gen;
use crate::piece;

const CENTRAL_CONTROL: [i16; 120] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 2,
    2, 1, 0, 0, 0, 0, 1, 2, 3, 3, 3, 3, 2, 1, 0, 0, 0, 0, 1, 2, 3, 4, 4, 3, 2, 1, 0, 0, 0, 0, 1, 2,
    3, 4, 4, 3, 2, 1, 0, 0, 0, 0, 1, 2, 3, 3, 3, 3, 2, 1, 0, 0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 1, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const PAWN: i16 = 1;
const KNIGHT: i16 = 3;
const BISHOP: i16 = 3;
const ROOK: i16 = 5;
const QUEEN: i16 = 9;

const PAWN_PROTECTED: i16 = 25;
const KNIGHT_PROTECTED: i16 = 15;
const BISHOP_PROTECTED: i16 = 15;
const ROOK_PROTECTED: i16 = 10;
const QUEEN_PROTECTED: i16 = 5;

const PAWN_PROTECION: i16 = 5;
const KNIGHT_PROTECION: i16 = 2;
const BISHOP_PROTECION: i16 = 3;
const ROOK_PROTECION: i16 = 3;
const QUEEN_PROTECION: i16 = 2;
const KING_PROTECTION: i16 = 5;

const PAWN_MOVEMENT: i16 = 2;
const KNIGHT_MOVEMENT: i16 = 3;
const BISHOP_MOVEMENT: i16 = 5;
const ROOK_MOVEMENT: i16 = 5;
const QUEEN_MOVEMENT: i16 = 9;
const KING_MOVEMENT: i16 = 1;

pub fn eval(game: &mut game::GameInfo) -> f64 {
    let white = material_eval(&game.white_pieces);
    let black = material_eval(&game.black_pieces);

    let score = 4.0 * ((white - black) as f64 / 39.0);
    score
}

pub fn net_eval(game: &mut game::GameInfo, net: &Net) -> f64 {
    net.forward(&model::pre_proccess(game))
        .f_double_value(&[0])
        .unwrap()
}

pub fn net_eval_tch(game: &mut game::GameInfo, net: &Net) -> tch::Tensor {
    net.forward(&model::pre_proccess(game))
}

fn material_eval(piece_list: &piece::PieceList) -> i16 {
    let mut score: i16 = 0;

    score += (piece_list.pawns.len() as i16) * PAWN;
    score += (piece_list.knights.len() as i16) * KNIGHT;
    score += (piece_list.bishops.len() as i16) * BISHOP;
    score += (piece_list.rooks.len() as i16) * ROOK;
    score += (piece_list.queens.len() as i16) * QUEEN;

    score
}

fn mobility_conectivity_eval(
    mobility: &mut HashMap<i8, i16>,
    conectivity: &mut HashMap<i8, i16>,
    moves: &Vec<move_gen::Move>,
    board: &[piece::Piece; 120],
    turn: &game::Color,
) -> i16 {
    let mut score: i16 = 0;

    for movement in moves {
        if mobility.contains_key(&movement.origin) {
            *mobility.get_mut(&movement.origin).unwrap() += 1;
        } else {
            mobility.insert(movement.origin, 1);
        }

        if conectivity.contains_key(&movement.origin) {
            match board[movement.destiny as usize] {
                piece::Piece::White(p) => {
                    *conectivity.get_mut(&movement.origin).unwrap() *= piece_type_protection(p)
                }
                piece::Piece::Black(p) => {
                    *conectivity.get_mut(&movement.origin).unwrap() *= piece_type_protection(p)
                }
                _ => (),
            }
        } else {
            let piece_type = match board[movement.origin as usize] {
                piece::Piece::White(p) => Some(p),
                piece::Piece::Black(p) => Some(p),
                _ => None,
            }
            .unwrap();

            conectivity.insert(movement.origin, piece_type_protected(piece_type));
        }
    }

    for key in mobility.keys() {
        let piece = match board[*key as usize] {
            piece::Piece::White(p) => Some(p),
            piece::Piece::Black(p) => Some(p),
            _ => None,
        }
        .unwrap();

        match piece {
            piece::PieceType::Pawn => score += PAWN_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::Knight => score += KNIGHT_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::Bishop => score += BISHOP_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::Rook => score += ROOK_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::Queen => score += QUEEN_MOVEMENT * mobility.get(key).unwrap(),
            piece::PieceType::King => score += KING_MOVEMENT * mobility.get(key).unwrap(),
        }
    }
    for value in conectivity.values() {
        score += *value;
    }

    score
}

fn piece_type_protected(piece: piece::PieceType) -> i16 {
    match piece {
        piece::PieceType::Pawn => PAWN_PROTECTED,
        piece::PieceType::Knight => KNIGHT_PROTECTED,
        piece::PieceType::Bishop => BISHOP_PROTECTED,
        piece::PieceType::Rook => ROOK_PROTECTED,
        piece::PieceType::Queen => QUEEN_PROTECTED,
        _ => 0,
    }
}

fn piece_type_protection(piece: piece::PieceType) -> i16 {
    match piece {
        piece::PieceType::Pawn => PAWN_PROTECION,
        piece::PieceType::Knight => KNIGHT_PROTECION,
        piece::PieceType::Bishop => BISHOP_PROTECION,
        piece::PieceType::Rook => ROOK_PROTECION,
        piece::PieceType::Queen => QUEEN_PROTECION,
        piece::PieceType::King => KING_PROTECTION,
    }
}

fn center_control_eval(attacks: &[u8; 120]) -> i16 {
    let mut score: i16 = 0;

    for (index, value) in attacks.iter().enumerate() {
        score += (*value as i16) * CENTRAL_CONTROL[index];
    }

    score
}

fn king_safety_eval() -> i16 {
    let mut score: i16 = 0;

    score
}
