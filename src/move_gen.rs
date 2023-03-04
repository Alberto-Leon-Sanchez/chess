use crate::attack_gen;
use crate::game;
use crate::piece;

pub const DIAGONAL_SLIDING: [i8; 4] = [9, 11, -11, -9];
pub const LATERAL_SLIDING: [i8; 4] = [10, 1, -10, -1];
pub const KNIGHT_SLIDING: [i8; 8] = [-21, -19, -12, -8, 8, 12, 19, 21];
const PROMOTION_TYPES: [piece::PieceType; 4] = [
    piece::PieceType::Queen,
    piece::PieceType::Rook,
    piece::PieceType::Bishop,
    piece::PieceType::Knight,
];

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub origin: i8,
    pub destiny: i8,
    pub destiny_piece: piece::Piece,
    pub promotion: Option<piece::PieceType>,
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.origin == other.origin
            && self.destiny == other.destiny
            && self.destiny_piece == other.destiny_piece
            && self.promotion == other.promotion
    }
}

impl Move {
    pub fn new() -> Move {
        Move {
            origin: 0,
            destiny: 0,
            destiny_piece: piece::Piece::Empty,
            promotion: None,
        }
    }
}

pub fn move_gen(game: &mut game::GameInfo) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let (attacks, attacker_pos) = attack_gen::attack_gen(game, None);

    let mut piece_list = &mut game.white_pieces;
    let mut opposite_piece_list = &mut game.black_pieces;
    if matches!(game.turn, game::Color::Black) {
        piece_list = &mut game.black_pieces;
        opposite_piece_list = &mut game.white_pieces;
    }

    let (pinned_piece_list, mut pinned_moves) = get_pinned_pieces(
        &mut game.board,
        *piece_list.kings.last().unwrap(),
        &game.turn,
    );
    piece_list.diference(&pinned_piece_list);

    if attacks[*piece_list.kings.last().unwrap() as usize] == 0 {
        moves.append(&mut direction_sliding(
            &piece_list.bishops,
            &game.board,
            &game.turn,
            &DIAGONAL_SLIDING,
            &Vec::new(),
        ));
        moves.append(&mut direction_sliding(
            &piece_list.queens,
            &game.board,
            &game.turn,
            &DIAGONAL_SLIDING,
            &Vec::new(),
        ));

        moves.append(&mut direction_sliding(
            &piece_list.rooks,
            &game.board,
            &game.turn,
            &LATERAL_SLIDING,
            &Vec::new(),
        ));
        moves.append(&mut direction_sliding(
            &piece_list.queens,
            &game.board,
            &game.turn,
            &LATERAL_SLIDING,
            &Vec::new(),
        ));

        moves.append(&mut knight_moves(
            &piece_list.knights,
            &game.board,
            &game.turn,
            &Vec::new(),
        ));

        moves.append(&mut pawn_moves(
            &piece_list.pawns,
            &mut game.board,
            &game.turn,
            game.en_passant.last().unwrap(),
            &Vec::new(),
            opposite_piece_list,
            *piece_list.kings.last().unwrap(),
        ));

        moves.append(&mut pinned_moves);
    } else if attacks[*piece_list.kings.last().unwrap() as usize] == 1 {
        let mut line: Vec<i8> = vec![-1];
        let mut piece: piece::PieceType = piece::PieceType::Rook;

        match game.board[attacker_pos as usize] {
            piece::Piece::White(p) => piece = p,
            piece::Piece::Black(p) => piece = p,
            _ => (),
        }

        if attacker_pos != 0 && !matches!(piece, piece::PieceType::Knight) {
            line = get_line(
                attacker_pos,
                *piece_list.kings.last().unwrap(),
                get_direction(attacker_pos, *piece_list.kings.last().unwrap()),
            );
        } else if matches!(piece, piece::PieceType::Knight) {
            line = vec![attacker_pos];
        }

        moves.append(&mut direction_sliding(
            &piece_list.bishops,
            &game.board,
            &game.turn,
            &DIAGONAL_SLIDING,
            &line,
        ));
        moves.append(&mut direction_sliding(
            &piece_list.queens,
            &game.board,
            &game.turn,
            &DIAGONAL_SLIDING,
            &line,
        ));

        moves.append(&mut direction_sliding(
            &piece_list.rooks,
            &game.board,
            &game.turn,
            &LATERAL_SLIDING,
            &line,
        ));
        moves.append(&mut direction_sliding(
            &piece_list.queens,
            &game.board,
            &game.turn,
            &LATERAL_SLIDING,
            &line,
        ));

        moves.append(&mut knight_moves(
            &piece_list.knights,
            &game.board,
            &game.turn,
            &line,
        ));

        moves.append(&mut pawn_moves(
            &piece_list.pawns,
            &mut game.board,
            &game.turn,
            game.en_passant.last().unwrap(),
            &line,
            opposite_piece_list,
            *piece_list.kings.last().unwrap(),
        ));
    }

    moves.append(&mut king_moves(
        &piece_list.kings,
        &game.board,
        &game.turn,
        game.castling.last().unwrap(),
        &attacks,
    ));

    piece_list.add(pinned_piece_list);

    moves
}

pub fn direction_sliding(
    piece_list: &Vec<i8>,
    board: &[piece::Piece; 120],
    turn: &game::Color,
    direction: &[i8],
    line: &Vec<i8>,
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    for piece in piece_list {
        for direction in direction {
            let mut destiny = *piece + direction;
            let mut destiny_piece = board[destiny as usize];

            loop {
                match destiny_piece {
                    piece::Piece::White(_) => {
                        if matches!(turn, game::Color::Black)
                            && (line.is_empty() || line.contains(&destiny))
                        {
                            moves.push(Move {
                                origin: *piece,
                                destiny,
                                destiny_piece,
                                promotion: None,
                            });
                        }
                        break;
                    }
                    piece::Piece::Black(_) => {
                        if matches!(*turn, game::Color::White)
                            && (line.is_empty() || line.contains(&destiny))
                        {
                            moves.push(Move {
                                origin: *piece,
                                destiny,
                                destiny_piece,
                                promotion: None,
                            });
                        }
                        break;
                    }
                    piece::Piece::Empty => {
                        if line.is_empty() || line.contains(&destiny) {
                            moves.push(Move {
                                origin: *piece,
                                destiny,
                                destiny_piece: piece::Piece::Empty,
                                promotion: None,
                            });
                        }
                        destiny += direction;
                        destiny_piece = board[destiny as usize];
                    }
                    piece::Piece::Outside => break,
                }
            }
        }
    }
    moves
}

pub fn knight_moves(
    piece_list: &Vec<i8>,
    board: &[piece::Piece; 120],
    turn: &game::Color,
    line: &Vec<i8>,
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    for piece in piece_list {
        for direction in KNIGHT_SLIDING {
            let destiny = *piece + direction;
            let destiny_piece = board[destiny as usize];

            match destiny_piece {
                piece::Piece::White(_) => {
                    if matches!(turn, game::Color::Black)
                        && (line.is_empty() || line.contains(&destiny))
                    {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
                piece::Piece::Black(_) => {
                    if matches!(*turn, game::Color::White)
                        && (line.is_empty() || line.contains(&destiny))
                    {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
                piece::Piece::Empty => {
                    if line.is_empty() || line.contains(&destiny) {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece: piece::Piece::Empty,
                            promotion: None,
                        });
                    }
                }
                piece::Piece::Outside => (),
            }
        }
    }
    moves
}

pub fn pawn_moves(
    piece_list: &Vec<i8>,
    board: &mut [piece::Piece; 120],
    turn: &game::Color,
    en_passant: &Option<i8>,
    line: &Vec<i8>,
    pieces: &piece::PieceList,
    king: i8,
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    let mut movement = 10;
    let mut square_difence: i8 = 10;

    if matches!(turn, game::Color::Black) {
        movement = -10;
        square_difence = -10;
    }

    for piece in piece_list {
        let mut destiny = *piece + movement;
        let mut destiny_piece = board[destiny as usize];
        match destiny_piece {
            piece::Piece::Empty => {
                if line.is_empty() || line.contains(&destiny) {
                    if !(29..=90).contains(&destiny) {
                        moves.append(&mut pawn_promotion(*piece, destiny, destiny_piece));
                    } else {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
                if !is_pawn_move(piece, turn) {
                    destiny = *piece + movement * 2;
                    destiny_piece = board[destiny as usize];
                    match destiny_piece {
                        piece::Piece::Empty => {
                            if line.is_empty() || line.contains(&destiny) {
                                moves.push(Move {
                                    origin: *piece,
                                    destiny,
                                    destiny_piece,
                                    promotion: None,
                                });
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => {}
        }

        destiny = *piece + movement - 1;
        destiny_piece = board[destiny as usize];
        match destiny_piece {
            piece::Piece::Black(_) => {
                if matches!(*turn, game::Color::White)
                    && (line.is_empty() || line.contains(&destiny))
                {
                    if !(29..=90).contains(&destiny) {
                        moves.append(&mut pawn_promotion(*piece, destiny, destiny_piece));
                    } else {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
            }
            piece::Piece::White(_) => {
                if matches!(*turn, game::Color::Black)
                    && (line.is_empty() || line.contains(&destiny))
                {
                    if !(29..=90).contains(&destiny) {
                        moves.append(&mut pawn_promotion(*piece, destiny, destiny_piece));
                    } else {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
            }
            _ => match en_passant {
                Some(pos) => {
                    if !discover_check(
                        board,
                        *piece,
                        *piece - 1,
                        &pieces.rooks,
                        &pieces.queens,
                        king,
                    ) && destiny == *pos
                        && (line.is_empty() || line.contains(&destiny) || line.len() == 2)
                    {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece: board[(destiny - square_difence) as usize],
                            promotion: None,
                        });
                    }
                }
                None => (),
            },
        }

        destiny = *piece + movement + 1;
        destiny_piece = board[destiny as usize];
        match destiny_piece {
            piece::Piece::Black(_) => {
                if matches!(*turn, game::Color::White)
                    && (line.is_empty() || line.contains(&destiny))
                {
                    if !(29..=90).contains(&destiny) {
                        moves.append(&mut pawn_promotion(*piece, destiny, destiny_piece));
                    } else {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
            }
            piece::Piece::White(_) => {
                if matches!(*turn, game::Color::Black)
                    && (line.is_empty() || line.contains(&destiny))
                {
                    if !(29..=90).contains(&destiny) {
                        moves.append(&mut pawn_promotion(*piece, destiny, destiny_piece));
                    } else {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
            }
            _ => match en_passant {
                Some(pos) => {
                    if !discover_check(
                        board,
                        *piece,
                        *piece + 1,
                        &pieces.rooks,
                        &pieces.queens,
                        king,
                    ) && destiny == *pos
                        && (line.is_empty() || line.contains(&destiny) || line.len() == 2)
                    {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece: board[(destiny - square_difence) as usize],
                            promotion: None,
                        });
                    }
                }
                None => (),
            },
        }
    }

    moves
}

pub fn is_pawn_move(pos: &i8, turn: &game::Color) -> bool {
    if matches!(*turn, game::Color::Black) {
        if *pos >= 81 && *pos <= 88 {
            return false;
        }
    } else if *pos >= 31 && *pos <= 38 {
        return false;
    }
    true
}

pub fn king_moves(
    piece_list: &Vec<i8>,
    board: &[piece::Piece; 120],
    turn: &game::Color,
    castling: &[bool; 4],
    attacks: &[u8; 120],
) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    for piece in piece_list {
        for direction in DIAGONAL_SLIDING.iter().chain(LATERAL_SLIDING.iter()) {
            let destiny = *piece + direction;
            let destiny_piece = board[destiny as usize];

            if attacks[destiny as usize] > 0 {
                continue;
            }

            match destiny_piece {
                piece::Piece::White(_) => {
                    if matches!(turn, game::Color::Black) {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
                piece::Piece::Black(_) => {
                    if matches!(*turn, game::Color::White) {
                        moves.push(Move {
                            origin: *piece,
                            destiny,
                            destiny_piece,
                            promotion: None,
                        });
                    }
                }
                piece::Piece::Empty => {
                    moves.push(Move {
                        origin: *piece,
                        destiny,
                        destiny_piece: piece::Piece::Empty,
                        promotion: None,
                    });
                }
                piece::Piece::Outside => {}
            }
        }
        if attacks[*piece as usize] == 0 {
            match turn {
                game::Color::White => {
                    if castling[0]
                        && is_empty(*piece + 1, board)
                        && is_empty(*piece + 2, board)
                        && attacks[(*piece + 1) as usize] == 0
                        && attacks[(*piece + 2) as usize] == 0
                    {
                        moves.push(Move {
                            origin: *piece,
                            destiny: *piece + 2,
                            destiny_piece: piece::Piece::Empty,
                            promotion: None,
                        })
                    }
                    if castling[1]
                        && is_empty(*piece - 1, board)
                        && is_empty(*piece - 2, board)
                        && is_empty(*piece - 3, board)
                        && attacks[(*piece - 1) as usize] == 0
                        && attacks[(*piece - 2) as usize] == 0
                    {
                        moves.push(Move {
                            origin: *piece,
                            destiny: *piece - 2,
                            destiny_piece: piece::Piece::Empty,
                            promotion: None,
                        })
                    }
                }
                game::Color::Black => {
                    if castling[2]
                        && is_empty(*piece + 1, board)
                        && is_empty(*piece + 2, board)
                        && attacks[(*piece + 1) as usize] == 0
                        && attacks[(*piece + 2) as usize] == 0
                    {
                        moves.push(Move {
                            origin: *piece,
                            destiny: *piece + 2,
                            destiny_piece: piece::Piece::Empty,
                            promotion: None,
                        })
                    }
                    if castling[3]
                        && is_empty(*piece - 1, board)
                        && is_empty(*piece - 2, board)
                        && is_empty(*piece - 3, board)
                        && attacks[(*piece - 1) as usize] == 0
                        && attacks[(*piece - 2) as usize] == 0
                    {
                        moves.push(Move {
                            origin: *piece,
                            destiny: *piece - 2,
                            destiny_piece: piece::Piece::Empty,
                            promotion: None,
                        })
                    }
                }
            }
        }
    }

    moves
}

fn is_empty(pos: i8, board: &[piece::Piece; 120]) -> bool {
    if matches!(board[pos as usize], piece::Piece::Empty) {
        true
    } else {
        false
    }
}

fn get_direction(origin: i8, destiny: i8) -> i8 {
    let sign: i8;
    let direction = (origin - destiny).abs();

    if origin - destiny < 0 {
        sign = 1;
    } else {
        sign = -1;
    }

    if direction < 8 {
        sign
    } else if direction % 10 == 0 {
        10 * sign
    } else if direction % 9 == 0 {
        9 * sign
    } else {
        11 * sign
    }
}

fn get_line(origin: i8, destiny: i8, direction: i8) -> Vec<i8> {
    let mut line: Vec<i8> = vec![-1];
    let mut pos = origin;

    while pos != destiny {
        line.push(pos);
        pos += direction;
    }

    line
}

fn get_pinned_pieces(
    board: &mut [piece::Piece; 120],
    king: i8,
    turn: &game::Color,
) -> (piece::PieceList, Vec<Move>) {
    let mut pinned_pieces = piece::PieceList::new();
    let mut piece: Option<i8> = None;
    let mut moves: Vec<Move> = Vec::new();
    let mut end: i8 = 0;

    for direction in DIAGONAL_SLIDING.iter().chain(LATERAL_SLIDING.iter()) {
        let mut destiny = king + direction;

        loop {
            match board[destiny as usize] {
                piece::Piece::White(p) => {
                    if piece.is_some() {
                        if matches!(turn, game::Color::White) {
                            piece = None;
                            break;
                        } else if p.is_sliding() && p.direction(*direction) {
                            end = destiny;
                            break;
                        } else {
                            piece = None;
                            break;
                        }
                    } else if matches!(turn, game::Color::White) {
                        piece = Some(destiny);
                    } else {
                        break;
                    }
                    destiny += direction
                }
                piece::Piece::Black(p) => {
                    if piece.is_some() {
                        if matches!(turn, game::Color::Black) {
                            piece = None;
                            break;
                        } else if p.is_sliding() && p.direction(*direction) {
                            end = destiny;
                            break;
                        } else {
                            piece = None;
                            break;
                        }
                    } else if matches!(turn, game::Color::Black) {
                        piece = Some(destiny);
                    } else {
                        break;
                    }
                    destiny += direction
                }
                piece::Piece::Empty => destiny += direction,
                piece::Piece::Outside => {
                    piece = None;
                    break;
                }
            }
        }

        if let Some(pos) = piece {
            let line = get_line(end, king, get_direction(end, king));

            let mut piece_type: piece::PieceType = piece::PieceType::Rook;
            match board[pos as usize] {
                piece::Piece::White(p) => {
                    pinned_pieces.add_piece(&p, &pos);
                    piece_type = p;
                }
                piece::Piece::Black(p) => {
                    pinned_pieces.add_piece(&p, &pos);
                    piece_type = p;
                }
                _ => (),
            }

            match piece_type {
                piece::PieceType::Rook => {
                    moves.append(&mut direction_sliding(
                        &vec![pos],
                        board,
                        turn,
                        &LATERAL_SLIDING,
                        &line,
                    ));
                }
                piece::PieceType::Bishop => {
                    moves.append(&mut direction_sliding(
                        &vec![pos],
                        board,
                        turn,
                        &DIAGONAL_SLIDING,
                        &line,
                    ));
                }
                piece::PieceType::Queen => {
                    moves.append(&mut direction_sliding(
                        &vec![pos],
                        board,
                        turn,
                        &DIAGONAL_SLIDING,
                        &line,
                    ));
                    moves.append(&mut direction_sliding(
                        &vec![pos],
                        board,
                        turn,
                        &LATERAL_SLIDING,
                        &line,
                    ));
                }
                piece::PieceType::Pawn => {
                    moves.append(&mut pawn_moves(
                        &vec![pos],
                        board,
                        turn,
                        &None,
                        &line,
                        &piece::PieceList::new(),
                        king,
                    ));
                }
                _ => (),
            }

            piece = None;
            end = 0;
        }
    }

    (pinned_pieces, moves)
}

fn discover_check(
    board: &mut [piece::Piece; 120],
    index: i8,
    index2: i8,
    rooks: &Vec<i8>,
    queens: &Vec<i8>,
    king: i8,
) -> bool {
    let p = board[index as usize];
    let p2 = board[index2 as usize];
    board[index as usize] = piece::Piece::Empty;
    board[index2 as usize] = piece::Piece::Empty;

    let mut attacks: [u8; 120] = [0; 120];

    attack_gen::direction_sliding(rooks, board, &[1, -1], &mut attacks, king);
    attack_gen::direction_sliding(queens, board, &[1, -1], &mut attacks, king);

    board[index as usize] = p;
    board[index2 as usize] = p2;

    if attacks[king as usize] > 0 {
        return true;
    }

    false
}

fn pawn_promotion(origin: i8, destiny: i8, destiny_piece: piece::Piece) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    for piece_type in PROMOTION_TYPES {
        moves.push(Move {
            origin,
            destiny,
            destiny_piece,
            promotion: Some(piece_type),
        });
    }

    moves
}
