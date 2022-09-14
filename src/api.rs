use http_types::convert::json;
use serde::{Deserialize,Serialize};
use tide;
use crate::fen_reader;
use crate::move_gen;
use crate::fen_writer;
use crate::make_move;
use crate::piece;

#[derive(Deserialize,Serialize)]
struct Fen{
    fen:String
}

#[derive(Serialize,Deserialize)]
struct Move{
    origin:i8,
    destiny:i8,
}

#[derive(Serialize,Deserialize)]
struct MoveFen{
    origin:i8,
    destiny:i8,
    fen:String
}

pub async fn get_moves(request:tide::Request<()>) -> tide::Result{

    let fen:Fen = request.query()?;
    let mut game = fen_reader::read_fen(&fen.fen);
    
    let moves = move_gen::move_gen(&mut game);
    let mut api_moves:Vec<Move> = Vec::new();

    for pos in moves{

        api_moves.push(Move{origin:board120_to_board64(pos.origin), destiny:board120_to_board64(pos.destiny)});
    }
    
    Ok(json!(api_moves).into())
}

pub async fn make_move(request:tide::Request<()>) -> tide::Result{

    let mut movement:MoveFen = request.query()?;
    
    let en_passant:Option<i8> = None;
    let mut game_info = fen_reader::read_fen(&movement.fen);
    
    let destiny_piece:piece::Piece;

    movement.origin = board64_to_board120(movement.origin);
    movement.destiny = board64_to_board120(movement.destiny);

    match en_passant{
        Some(pos) => {
            destiny_piece = game_info.board[pos as usize];
        },
        None => {
            destiny_piece = game_info.board[movement.destiny as usize];
        },
    }
    
    let mut movement:move_gen::Move = move_gen::Move{origin:movement.origin, destiny:movement.destiny, destiny_piece};
    
    game_info = make_move::make_move(game_info, &mut movement);
    game_info.print_board();
    let fen:String = fen_writer::write_fen(&game_info);
    println!("{}",fen);
    
    Ok(json!(Fen{fen:fen}).into())

}


fn board64_to_board120(pos:i8) -> i8{
    let mut row = pos/8;
    let mut col = pos%8;
    row = row * 10;
    
    row + col + fen_reader::ROW_OFF_SET as i8 + fen_reader::MAILBOX_OFF_SET as i8
}

fn board120_to_board64(pos:i8) -> i8{

    let mut row = pos / 10 - 2;
    let col = pos % 10;
    row = row*8 + col - 1;
    
    row
}