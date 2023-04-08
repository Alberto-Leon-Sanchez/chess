use std::fs::File;
use std::io::BufReader;

use crate::alpha_beta_search;
use crate::fen_reader;
use crate::fen_writer;
use crate::make_move;
use crate::model;
use crate::move_gen;
use crate::piece;
use http_types::convert::json;
use serde::{Deserialize, Serialize};
use tide;

#[derive(Deserialize, Serialize)]
struct Fen {
    fen: String,
}

#[derive(Serialize, Deserialize)]
struct Move {
    origin: i8,
    destiny: i8,
}

#[derive(Serialize, Deserialize)]
struct MoveFen {
    origin: i8,
    destiny: i8,
    promotion: String,
    fen: String,
}

#[derive(Deserialize)]
struct BestMove {
    fen: String,
    depth: i8,
}

pub async fn get_moves(request: tide::Request<()>) -> tide::Result {
    let fen: Fen = request.query()?;
    let mut game = fen_reader::read_fen(&fen.fen);

    let moves = move_gen::move_gen(&mut game);
    let mut api_moves: Vec<Move> = Vec::new();

    for pos in moves {
        api_moves.push(Move {
            origin: board120_to_board64(pos.origin),
            destiny: board120_to_board64(pos.destiny),
        });
    }

    Ok(json!(api_moves).into())
}

pub async fn make_move(request: tide::Request<()>) -> tide::Result {
    let mut movement: MoveFen = request.query()?;

    let en_passant: Option<i8> = None;
    let mut game_info = fen_reader::read_fen(&movement.fen);

    let destiny_piece: piece::Piece;

    movement.origin = board64_to_board120(movement.origin);
    movement.destiny = board64_to_board120(movement.destiny);

    match en_passant {
        Some(pos) => {
            destiny_piece = game_info.board[pos as usize];
        }
        None => {
            destiny_piece = game_info.board[movement.destiny as usize];
        }
    }

    let promotion: Option<piece::PieceType> = letter_to_piece(movement.promotion);

    let mut movement: move_gen::Move = move_gen::Move {
        origin: movement.origin,
        destiny: movement.destiny,
        destiny_piece,
        promotion,
    };

    make_move::make_move(&mut game_info, &mut movement);
    game_info.print_board();
    let fen: String = fen_writer::write_fen(&game_info);
    println!("{}", fen);

    Ok(json!(Fen { fen }).into())
}

pub async fn get_best(request: tide::Request<()>) -> tide::Result {
    let query: BestMove = request.query()?;
    let depth = query.depth;
    let mut game = fen_reader::read_fen(&query.fen);

    let mut vs = tch::nn::VarStore::new(tch::Device::Cpu);
    let net = model::model(vs.root());
    vs.load_from_stream(&mut BufReader::new(
        File::open("./model_weights/3_hidden_1014.pt").unwrap(),
    ))
    .unwrap();

    let mut movement = alpha_beta_search::get_best(&mut game, depth, &net);
    make_move::make_move(&mut game, &mut movement);
    let fen = fen_writer::write_fen(&game);

    Ok(json!(Fen { fen }).into())
}

pub fn board64_to_board120(pos: i8) -> i8 {
    let mut row = pos / 8;
    let col = pos % 8;
    row *= 10;

    row + col + fen_reader::ROW_OFF_SET as i8 + fen_reader::MAILBOX_OFF_SET as i8
}

pub fn board120_to_board64(pos: i8) -> i8 {
    let mut row = pos / 10 - 2;
    let col = pos % 10;
    row = row * 8 + col - 1;

    row
}

fn letter_to_piece(mut letter: String) -> Option<piece::PieceType> {
    letter = letter.to_ascii_uppercase();

    match letter.as_ref() {
        "P" => Some(piece::PieceType::Pawn),
        "N" => Some(piece::PieceType::Knight),
        "B" => Some(piece::PieceType::Bishop),
        "R" => Some(piece::PieceType::Rook),
        "Q" => Some(piece::PieceType::Queen),
        "K" => Some(piece::PieceType::King),
        _ => None,
    }
}

