use crate::game;
use crate::piece;

const PAWN:i8 = 1;
const KNIGHT:i8 = 3;
const BISHOP:i8 = 3;
const ROOK:i8 = 5;
const QUEEN:i8 = 9;
const KING:i8 = 100;

pub fn eval(game:&game::GameInfo) -> i8{

    let mut score = 0;

    let turn:i8 = if game.turn == game::Color::White{
        1
    }else{
        -1
    };

    let piece_list = if game.turn == game::Color::White{
        &game.white_pieces
    }else{
        &game.black_pieces
    };

    score += material_eval(piece_list);


    score * turn
}

fn material_eval(piece_list:&piece::PieceList) ->i8{
    let mut score:i8 = 0;
    
    score += (piece_list.pawns.len() as i8) * PAWN;
    score += (piece_list.knights.len() as i8) * KNIGHT;
    score += (piece_list.bishops.len() as i8) * BISHOP;
    score += (piece_list.rooks.len() as i8) * ROOK;
    score += (piece_list.queens.len() as i8) * QUEEN;
    score += (piece_list.kings.len() as i8) * KING;

    return score;
}

fn movility_eval(piece_list:&piece::PieceList) -> i8{
    let mut score:i8 = 0;

    

    return score;
}