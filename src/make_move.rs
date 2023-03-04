use crate::game;
use crate::move_gen;
use crate::piece;
use crate::zobrist_hashing;

pub fn make_move(game: &mut game::GameInfo, movement: &mut move_gen::Move) {
    let piece: piece::Piece = game.board[movement.origin as usize];
    match piece {
        piece::Piece::Empty => {
            game.print_board();
            panic!("Trying to move an empty square");
        }
        piece::Piece::Outside => panic!("Trying to move an outside square"),
        piece::Piece::White(piece) => aux(game, movement, piece),
        piece::Piece::Black(piece) => aux(game, movement, piece),
    }
}

fn aux(mut game: &mut game::GameInfo, movement: &mut move_gen::Move, mut piece: piece::PieceType) {
    unsafe {
        zobrist_hashing::HASH.hash_move(piece, &mut game.hash, movement.origin, &game.turn);
    }

    if let Some(piece_type) = movement.promotion {
        piece = piece_type;

        match game.turn {
            game::Color::White => {
                game.white_pieces
                    .remove(&piece::PieceType::Pawn, movement.origin);
                game.white_pieces.add_piece(&piece, &movement.destiny);
            }
            game::Color::Black => {
                game.black_pieces
                    .remove(&piece::PieceType::Pawn, movement.origin);
                game.black_pieces.add_piece(&piece, &movement.destiny);
            }
        }
    }

    unsafe {
        zobrist_hashing::HASH.hash_move(piece, &mut game.hash, movement.destiny, &game.turn);
    }

    let mut square_diference: i8 = 10;

    match game.turn {
        game::Color::White => {
            game.board[movement.destiny as usize] = piece::Piece::White(piece);
            game.white_pieces
                .make_move(&piece, movement.destiny, movement.origin);
        }
        game::Color::Black => {
            game.board[movement.destiny as usize] = piece::Piece::Black(piece);
            game.black_pieces
                .make_move(&piece, movement.destiny, movement.origin);
            square_diference = -10;
        }
    }

    if let Some(pos) = game.en_passant.last().unwrap() {
        if *pos == movement.destiny && matches!(piece, piece::PieceType::Pawn) {
            game.board[(*pos - square_diference) as usize] = piece::Piece::Empty;

            match game.turn {
                game::Color::White => {
                    game.black_pieces
                        .remove(&piece::PieceType::Pawn, *pos - square_diference);

                    unsafe {
                        zobrist_hashing::HASH.hash_move(
                            piece::PieceType::Pawn,
                            &mut game.hash,
                            *pos - square_diference,
                            &game::Color::Black,
                        );
                    }
                }
                game::Color::Black => {
                    game.white_pieces
                        .remove(&piece::PieceType::Pawn, *pos - square_diference);

                    unsafe {
                        zobrist_hashing::HASH.hash_move(
                            piece::PieceType::Pawn,
                            &mut game.hash,
                            *pos - square_diference,
                            &game::Color::White,
                        );
                    }
                }
            }
        } else {
            match movement.destiny_piece {
                piece::Piece::White(p) => unsafe {
                    zobrist_hashing::HASH.hash_move(
                        p,
                        &mut game.hash,
                        movement.destiny,
                        &game::Color::White,
                    )
                },
                piece::Piece::Black(p) => unsafe {
                    zobrist_hashing::HASH.hash_move(
                        p,
                        &mut game.hash,
                        movement.destiny,
                        &game::Color::Black,
                    )
                },
                _ => (),
            }
        }
    } else {
        match movement.destiny_piece {
            piece::Piece::White(p) => unsafe {
                zobrist_hashing::HASH.hash_move(
                    p,
                    &mut game.hash,
                    movement.destiny,
                    &game::Color::White,
                )
            },
            piece::Piece::Black(p) => unsafe {
                zobrist_hashing::HASH.hash_move(
                    p,
                    &mut game.hash,
                    movement.destiny,
                    &game::Color::Black,
                )
            },
            _ => (),
        }
    }

    if let piece::PieceType::King = piece {
        if (movement.origin - movement.destiny).abs() == 2 {
            match game.turn {
                game::Color::White => {
                    if movement.origin - movement.destiny < 0 {
                        game.board[26] = game.board[28];
                        game.board[28] = piece::Piece::Empty;
                        game.white_pieces.make_move(&piece::PieceType::Rook, 26, 28);

                        unsafe {
                            zobrist_hashing::HASH.hash_move(
                                piece::PieceType::Rook,
                                &mut game.hash,
                                28,
                                &game::Color::White,
                            );
                            zobrist_hashing::HASH.hash_move(
                                piece::PieceType::Rook,
                                &mut game.hash,
                                26,
                                &game::Color::White,
                            );
                        }
                    } else {
                        game.board[24] = game.board[21];
                        game.board[21] = piece::Piece::Empty;
                        game.white_pieces.make_move(&piece::PieceType::Rook, 24, 21);

                        unsafe {
                            zobrist_hashing::HASH.hash_move(
                                piece::PieceType::Rook,
                                &mut game.hash,
                                21,
                                &game::Color::White,
                            );
                            zobrist_hashing::HASH.hash_move(
                                piece::PieceType::Rook,
                                &mut game.hash,
                                24,
                                &game::Color::White,
                            );
                        }
                    }
                }
                game::Color::Black => {
                    if movement.origin - movement.destiny < 0 {
                        game.board[96] = game.board[98];
                        game.board[98] = piece::Piece::Empty;
                        game.black_pieces.make_move(&piece::PieceType::Rook, 96, 98);

                        unsafe {
                            zobrist_hashing::HASH.hash_move(
                                piece::PieceType::Rook,
                                &mut game.hash,
                                98,
                                &game::Color::Black,
                            );
                            zobrist_hashing::HASH.hash_move(
                                piece::PieceType::Rook,
                                &mut game.hash,
                                96,
                                &game::Color::Black,
                            );
                        }
                    } else {
                        game.board[94] = game.board[91];
                        game.board[91] = piece::Piece::Empty;
                        game.black_pieces.make_move(&piece::PieceType::Rook, 94, 91);

                        unsafe {
                            zobrist_hashing::HASH.hash_move(
                                piece::PieceType::Rook,
                                &mut game.hash,
                                91,
                                &game::Color::Black,
                            );
                            zobrist_hashing::HASH.hash_move(
                                piece::PieceType::Rook,
                                &mut game.hash,
                                94,
                                &game::Color::Black,
                            );
                        }
                    }
                }
            }
        }
    }

    game.board[movement.origin as usize] = piece::Piece::Empty;

    update_game_state(game, piece, movement);
}

fn update_game_state(
    mut game: &mut game::GameInfo,
    origin_piece: piece::PieceType,
    movement: &move_gen::Move,
) {
    let mut castling = *game.castling.last().unwrap();

    match movement.destiny_piece {
        piece::Piece::White(piece) => match piece {
            piece::PieceType::Rook => {
                if movement.destiny == 28 && castling[0] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 0) }
                    castling[0] = false;
                } else if movement.destiny == 21 && castling[1] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 1) }
                    castling[1] = false;
                }
            }
            _ => (),
        },
        piece::Piece::Black(piece) => match piece {
            piece::PieceType::Rook => {
                if movement.destiny == 98 && castling[2] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 2) }
                    castling[2] = false;
                } else if movement.destiny == 91 && castling[3] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 3) }
                    castling[3] = false;
                }
            }
            _ => (),
        },
        piece::Piece::Empty => (),
        piece::Piece::Outside => (),
    }

    match origin_piece {
        piece::PieceType::Rook => match game.turn {
            game::Color::White => {
                if movement.origin == 21 && castling[1] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 1) }
                    castling[1] = false;
                } else if movement.origin == 28 && castling[0] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 0) }
                    castling[0] = false;
                }
            }
            game::Color::Black => {
                if movement.origin == 91 && castling[3] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 3) }
                    castling[3] = false;
                } else if movement.origin == 98 && castling[2] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 2) }
                    castling[2] = false;
                }
            }
        },
        piece::PieceType::King => match game.turn {
            game::Color::White => {
                if castling[0] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 0) }
                    castling[0] = false;
                }
                if castling[1] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 1) }
                    castling[1] = false;
                }
            }
            game::Color::Black => {
                if castling[2] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 2) }
                    castling[2] = false;
                }
                if castling[3] {
                    unsafe { zobrist_hashing::HASH.hash_castling(&mut game.hash, 3) }
                    castling[3] = false;
                }
            }
        },
        _ => (),
    }

    if matches!(origin_piece, piece::PieceType::Pawn)
        && (movement.destiny - movement.origin).abs() > 12
    {
        let mut square_difference: i8 = 10;

        if matches!(game.turn, game::Color::Black) {
            square_difference = -10;
        }

        game.en_passant
            .push(Some(movement.destiny - square_difference));

        unsafe {
            zobrist_hashing::HASH
                .hash_en_passant(&mut game.hash, movement.destiny - square_difference)
        };
    } else {
        game.en_passant.push(None);
    }

    if !matches!(movement.destiny_piece, piece::Piece::Empty) {
        game.half_move_clock.push(0);

        match movement.destiny_piece {
            piece::Piece::White(piece) => game.white_pieces.remove(&piece, movement.destiny),
            piece::Piece::Black(piece) => game.black_pieces.remove(&piece, movement.destiny),
            _ => (),
        }
    } else if !matches!(origin_piece, piece::PieceType::Pawn) {
        game.half_move_clock
            .push(game.half_move_clock.last().unwrap() + 1);
    } else {
        game.half_move_clock.push(0);
    }

    game.castling.push(castling);

    if matches!(game.turn, game::Color::Black) {
        game.full_move += 1;
    }

    game.turn = game.turn.change_turn();
    unsafe { zobrist_hashing::HASH.hash_turn(&mut game.hash) }
}
