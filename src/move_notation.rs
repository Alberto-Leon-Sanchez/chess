use crate::{fen_writer, move_gen, piece};

pub fn get_move_notation(movement: &move_gen::Move, board: [piece::Piece; 120]) -> String {
    let mut white = true;

    match board[movement.destiny as usize] {
        piece::Piece::White(p) => p,
        piece::Piece::Black(p) => {
            white = false;
            p
        }
        _ => panic!("empty or outside"),
    };

    let mut move_notation = fen_writer::index_to_letter_pos(&movement.origin);
    /*
    if !matches!(piece, piece::PieceType::Pawn){
        move_notation = fen_writer::piece_to_letter(move_notation, &piece, white);
    }
    */
    move_notation += &fen_writer::index_to_letter_pos(&movement.destiny);

    match movement.promotion {
        Some(p) => move_notation = fen_writer::piece_to_letter(move_notation, &p, white),
        None => (),
    }

    move_notation.to_owned()
}
