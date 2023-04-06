use crate::game;
use crate::move_gen::{DIAGONAL_SLIDING, KNIGHT_SLIDING, LATERAL_SLIDING};
use crate::piece;

pub fn attack_gen(game: &mut game::GameInfo, color: Option<&game::Color>) -> ([u8; 120], i8) {
    let mut attacks: [u8; 120] = [0; 120];
    let mut attacker_pos: i8 = 0;

    let mut piece_list = &game.white_pieces;
    let mut king_pos = game.black_pieces.kings.last().unwrap();

    let mut turn = game::Color::White;
    
    match color {
        Some(color) if matches!(*color, game::Color::Black) => {
            piece_list = &game.black_pieces;
            king_pos = game.white_pieces.kings.last().unwrap();
        }
        None => {
            if !matches!(game.turn, game::Color::Black) {
                piece_list = &game.black_pieces;
                king_pos = game.white_pieces.kings.last().unwrap();
                turn = game::Color::Black;
            }
        }
        _ => (),
    }

    let king = game.board[*king_pos as usize];
    game.board[*king_pos as usize] = piece::Piece::Empty;
    let mut temp: i8;

    temp = direction_sliding(
        &piece_list.bishops,
        &game.board,
        &DIAGONAL_SLIDING,
        &mut attacks,
        *king_pos,
    );

    if temp != 0 {
        attacker_pos = temp;
    }

    temp = direction_sliding(
        &piece_list.queens,
        &game.board,
        &DIAGONAL_SLIDING,
        &mut attacks,
        *king_pos,
    );

    if temp != 0 {
        attacker_pos = temp;
    }

    temp = direction_sliding(
        &piece_list.rooks,
        &game.board,
        &LATERAL_SLIDING,
        &mut attacks,
        *king_pos,
    );

    if temp != 0 {
        attacker_pos = temp;
    }

    temp = direction_sliding(
        &piece_list.queens,
        &game.board,
        &LATERAL_SLIDING,
        &mut attacks,
        *king_pos,
    );

    if temp != 0 {
        attacker_pos = temp;
    }

    temp = knight_moves(&piece_list.knights, &game.board, &mut attacks, *king_pos);

    if temp != 0 {
        attacker_pos = temp;
    }

    temp = pawn_moves(
        &piece_list.pawns,
        &game.board,
        &turn,
        &mut attacks,
        *king_pos,
    );

    if temp != 0 {
        attacker_pos = temp;
    }

    king_moves(&piece_list.kings, &game.board, &mut attacks);

    game.board[*king_pos as usize] = king;

    (attacks, attacker_pos)
}

pub fn direction_sliding(
    piece_list: &Vec<i8>,
    board: &[piece::Piece; 120],
    direction: &[i8],
    attacks: &mut [u8; 120],
    king_pos: i8,
) -> i8 {
    let mut attacker_pos: i8 = 0;

    for piece in piece_list {
        for direction in direction {
            let mut destiny = *piece + direction;
            let mut destiny_piece = board[destiny as usize];

            loop {
                if destiny == king_pos {
                    attacker_pos = *piece;
                }
                match destiny_piece {
                    piece::Piece::Empty => {
                        attacks[destiny as usize] += 1;
                        destiny += direction;
                        destiny_piece = board[destiny as usize];
                    }
                    piece::Piece::Outside => break,
                    _ => {
                        attacks[destiny as usize] += 1;
                        break;
                    }
                }
            }
        }
    }

    attacker_pos
}

fn knight_moves(
    piece_list: &Vec<i8>,
    board: &[piece::Piece; 120],
    attacks: &mut [u8; 120],
    king_pos: i8,
) -> i8 {
    let mut attacker_pos: i8 = 0;

    for piece in piece_list {
        for direction in KNIGHT_SLIDING {
            let destiny = *piece + direction;
            let destiny_piece = board[destiny as usize];

            match destiny_piece {
                piece::Piece::Outside => (),
                _ => {
                    attacks[destiny as usize] += 1;
                }
            }

            if destiny == king_pos {
                attacker_pos = *piece;
            }
        }
    }

    attacker_pos
}
fn pawn_moves(
    piece_list: &Vec<i8>,
    board: &[piece::Piece; 120],
    turn: &game::Color,
    attacks: &mut [u8; 120],
    king_pos: i8,
) -> i8 {
    let mut movement = 10;
    let mut attacker_pos: i8 = 0;

    if matches!(turn, game::Color::Black) {
        movement = -10;
    }

    for piece in piece_list {
        let mut destiny = *piece + movement - 1;
        let mut destiny_piece = board[destiny as usize];
        match destiny_piece {
            piece::Piece::Outside => (),
            _ => attacks[destiny as usize] += 1,
        }

        if destiny == king_pos {
            attacker_pos = *piece;
        }

        destiny = *piece + movement + 1;
        destiny_piece = board[destiny as usize];
        match destiny_piece {
            piece::Piece::Outside => (),
            _ => attacks[destiny as usize] += 1,
        }

        if destiny == king_pos {
            attacker_pos = *piece;
        }
    }

    attacker_pos
}

fn king_moves(piece_list: &Vec<i8>, board: &[piece::Piece; 120], attacks: &mut [u8; 120]) {
    for piece in piece_list {
        for direction in DIAGONAL_SLIDING.iter().chain(LATERAL_SLIDING.iter()) {
            let destiny = *piece + direction;
            let destiny_piece = board[destiny as usize];
            match destiny_piece {
                piece::Piece::Outside => {}
                _ => {
                    attacks[destiny as usize] += 1;
                }
            }
        }
    }
}
