use crate::game;
use crate::move_gen;
use crate::piece;
use crate::zobrist_hashing;

pub fn unmake_move(mut game: &mut game::GameInfo, movement: move_gen::Move){
    if matches!(game.turn, game::Color::White) {
        game.full_move -= 1;
    }

    game.turn = game.turn.change_turn();
    unsafe{zobrist_hashing::HASH.hash_turn(&mut game.hash)}

    let rights = game.castling.pop().unwrap();
    let actual_rights = game.castling.last().unwrap();
    game.half_move_clock.pop();
    
    if let Some(pos)  = game.en_passant.pop().unwrap(){
        unsafe{ zobrist_hashing::HASH.hash_en_passant(&mut game.hash, pos)}
    }


    if !rights[0] && actual_rights[0]{
        unsafe{ zobrist_hashing::HASH.hash_castling(&mut game.hash, 0);}
    }
    if !rights[1] && actual_rights[1]{
        unsafe{ zobrist_hashing::HASH.hash_castling(&mut game.hash, 1);}
    }
    if !rights[2] && actual_rights[2]{
        unsafe{ zobrist_hashing::HASH.hash_castling(&mut game.hash, 2);}
    }
    if !rights[3] && actual_rights[3]{
        unsafe{ zobrist_hashing::HASH.hash_castling(&mut game.hash, 3);}
    }

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

                        unsafe{ 
                            zobrist_hashing::HASH.hash_move(piece::PieceType::Rook,&mut game.hash, 21, &game::Color::White);
                            zobrist_hashing::HASH.hash_move(piece::PieceType::Rook,&mut game.hash, 24, &game::Color::White);
                        }
                    } else {
                        game.board[28] = game.board[26];
                        game.board[26] = piece::Piece::Empty;
                        game.white_pieces.make_move(&piece::PieceType::Rook, 28, 26);

                        unsafe{ 
                            zobrist_hashing::HASH.hash_move(piece::PieceType::Rook,&mut game.hash, 28, &game::Color::White);
                            zobrist_hashing::HASH.hash_move(piece::PieceType::Rook,&mut game.hash, 26, &game::Color::White);
                        }
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

                        unsafe{ 
                            zobrist_hashing::HASH.hash_move(piece::PieceType::Rook,&mut game.hash, 91, &game::Color::Black);
                            zobrist_hashing::HASH.hash_move(piece::PieceType::Rook,&mut game.hash, 94, &game::Color::Black);
                        }
                    } else {
                        game.board[98] = game.board[96];
                        game.board[96] = piece::Piece::Empty;
                        game.black_pieces.make_move(&piece::PieceType::Rook, 98, 96);

                        unsafe{ 
                            zobrist_hashing::HASH.hash_move(piece::PieceType::Rook,&mut game.hash, 98, &game::Color::Black);
                            zobrist_hashing::HASH.hash_move(piece::PieceType::Rook,&mut game.hash, 96, &game::Color::Black);
                        }
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

                unsafe{ 
                    zobrist_hashing::HASH.hash_move(piece_type,&mut game.hash, movement.destiny, &game::Color::Black);
                    zobrist_hashing::HASH.hash_move(piece::PieceType::Pawn,&mut game.hash, movement.origin, &game::Color::Black);
                }
            }
            game::Color::White => {
                game.white_pieces.remove(&piece_type, movement.destiny);
                game.board[movement.origin as usize] = piece::Piece::White(piece::PieceType::Pawn);
                game.white_pieces
                    .add_piece(&piece::PieceType::Pawn, &movement.origin);

                unsafe{ 
                    zobrist_hashing::HASH.hash_move(piece_type,&mut game.hash, movement.destiny, &game::Color::White);
                    zobrist_hashing::HASH.hash_move(piece::PieceType::Pawn,&mut game.hash, movement.origin, &game::Color::White);
                }
            }
        }
    } else {
        unsafe{ 
            zobrist_hashing::HASH.hash_move(origin_piece,&mut game.hash, movement.origin, &game.turn);
            zobrist_hashing::HASH.hash_move(origin_piece,&mut game.hash, movement.destiny, &game.turn);
        }
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

                unsafe{ 
                    zobrist_hashing::HASH.hash_move(piece::PieceType::Pawn,&mut game.hash, *pos-square_difference, &game.turn.opposite_color());
                }
            }
            _ => {
                game.board[movement.destiny as usize] = movement.destiny_piece;
                match movement.destiny_piece{
                    piece::Piece::White(p) => unsafe{ 
                        zobrist_hashing::HASH.hash_move(p,&mut game.hash, movement.destiny, &game::Color::White);
                    },
                    piece::Piece::Black(p) => unsafe{ 
                        zobrist_hashing::HASH.hash_move(p,&mut game.hash, movement.destiny, &game::Color::Black);
                    },
                    _ => (),
                }
            },
        },
        None => {
            game.board[movement.destiny as usize] = movement.destiny_piece;
            match movement.destiny_piece{
                piece::Piece::White(p) => unsafe{ 
                    zobrist_hashing::HASH.hash_move(p,&mut game.hash, movement.destiny, &game::Color::White);
                },
                piece::Piece::Black(p) => unsafe{ 
                    zobrist_hashing::HASH.hash_move(p,&mut game.hash, movement.destiny, &game::Color::Black);
                },
                _ => (),
            }
        },
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

}
