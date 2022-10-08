use crate::game;
use crate::move_gen;
use crate::piece;

pub fn unmake_move(mut game: game::GameInfo, movement: move_gen::Move) -> game::GameInfo {
    if matches!(game.turn, game::Color::White) {
        game.full_move -= 1;
    }

    game.turn = game.turn.change_turn();
    game.castling.pop();
    game.half_move_clock.pop();
    game.en_passant.pop();

    let origin_piece = match game.board[movement.destiny as usize] {
        piece::Piece::White(p) => p,
        piece::Piece::Black(p) => p,
        _ => {
            game.print_board();
            panic!("there should be a piece")
        }
    };

    match game.board[movement.destiny as usize] {
        piece::Piece::White(p) => {
            if let piece::PieceType::King = p {
                if (movement.origin - movement.destiny).abs() == 2 {
                    if movement.origin - movement.destiny > 0 {
                        game.board[21] = game.board[24];
                        game.board[24] = piece::Piece::Empty;
                        game.white_pieces.make_move(&piece::PieceType::Rook, 21, 24);
                    } else {
                        game.board[28] = game.board[26];
                        game.board[26] = piece::Piece::Empty;
                        game.white_pieces.make_move(&piece::PieceType::Rook, 28, 26);
                    }
                }
            }
        }
        piece::Piece::Black(p) => {
            if let piece::PieceType::King = p {
                if (movement.origin - movement.destiny).abs() == 2 {
                    if movement.origin - movement.destiny > 0 {
                        game.board[91] = game.board[94];
                        game.board[94] = piece::Piece::Empty;
                        game.black_pieces.make_move(&piece::PieceType::Rook, 91, 94);
                    } else {
                        game.board[98] = game.board[96];
                        game.board[96] = piece::Piece::Empty;
                        game.black_pieces.make_move(&piece::PieceType::Rook, 98, 96);
                    }
                }
            }
        }
        _ => (),
    }

    if let Some(piece_type) = movement.promotion {
        match game.turn {
            game::Color::Black => {
                game.black_pieces.remove(&piece_type, movement.destiny);
                game.board[movement.origin as usize] = piece::Piece::Black(piece::PieceType::Pawn);
                game.black_pieces
                    .add_piece(&piece::PieceType::Pawn, &movement.origin);
            }
            game::Color::White => {
                game.white_pieces.remove(&piece_type, movement.destiny);
                game.board[movement.origin as usize] = piece::Piece::White(piece::PieceType::Pawn);
                game.white_pieces
                    .add_piece(&piece::PieceType::Pawn, &movement.origin);
            }
        }
    } else {
        game.board[movement.origin as usize] = game.board[movement.destiny as usize];
    }

    let square_difference: i8 = match game.turn {
        game::Color::White => 10,
        game::Color::Black => -10,
    };

    let mut destiny = movement.destiny;

    match game.en_passant.last() {
        Some(en_passant) => match en_passant {
            Some(pos)
                if matches!(origin_piece, piece::PieceType::Pawn) && movement.destiny == *pos =>
            {
                game.board[(*pos - square_difference) as usize] = movement.destiny_piece;
                game.board[movement.destiny as usize] = piece::Piece::Empty;
                destiny = *pos - square_difference;
            }
            _ => game.board[movement.destiny as usize] = movement.destiny_piece,
        },
        None => game.board[movement.destiny as usize] = movement.destiny_piece,
    }

    match movement.destiny_piece {
        piece::Piece::White(p) => game.white_pieces.add_piece(&p, &destiny),
        piece::Piece::Black(p) => game.black_pieces.add_piece(&p, &destiny),
        _ => (),
    }

    match game.board[movement.origin as usize] {
        piece::Piece::White(p) => {
            game.white_pieces
                .make_move(&p, movement.origin, movement.destiny)
        }
        piece::Piece::Black(p) => {
            game.black_pieces
                .make_move(&p, movement.origin, movement.destiny)
        }
        _ => (),
    }

    game
}
