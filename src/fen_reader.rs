use crate::game;
use crate::piece::Piece;
use crate::piece::PieceList;
use crate::piece::PieceType;
use crate::zobrist_hashing::HASH;

pub const MAILBOX_OFF_SET: u32 = 20;
pub const ROW_SIZE: u32 = 10;
pub const ROW_OFF_SET: u32 = 1;
const DECIMAL_RADIX: u32 = 10;

pub fn read_fen(fen: &str) -> game::GameInfo {
    let split: Vec<&str> = fen.split(' ').collect();
    let mut board: [Piece; 120] = [Piece::Outside; 120];

    complete_board(&mut board);
    set_pieces(&mut board, split[0]);

    let (white_pieces, black_pieces) = get_piece_lists(&board);

    let turn = get_turn(split[1]);

    let castling = get_castling(split[2]);

    let en_passant = get_en_passant(split[3]);

    let half_move_clock = get_half_move_clock(split[4]);

    let full_move;
    if split.len() == 6 {
        full_move = get_full_move(split[5]);
    } else {
        full_move = 0;
    }
    let hash;

    unsafe {
        hash = HASH.get_hash(
            &black_pieces,
            &white_pieces,
            &turn,
            &castling.last().unwrap(),
            &en_passant.last().unwrap(),
        );
    }

    game::GameInfo {
        board,
        white_pieces,
        black_pieces,
        turn,
        castling,
        en_passant,
        half_move_clock,
        full_move,
        hash,
        transposition_table: vec![game::Eval::new(); (game::TRANSPOSITION_TABLE_SIZE + 1) as usize],
    }
}

fn complete_board(board: &mut [Piece]) {
    for i in 0..8 {
        for j in 0..8 {
            board[row_column_to_index(&i, &j)] = Piece::Empty;
        }
    }
}

fn set_pieces(board: &mut [Piece], fen: &str) {
    let mut row = 7;
    let mut column = 0;

    for character in fen.chars() {
        match character {
            '/' => {
                row -= 1;
                column = 0;
            }
            '0'..='8' if column < 8 => column += character.to_digit(DECIMAL_RADIX).unwrap(),
            'A'..='z' if column < 8 => {
                board[row_column_to_index(&row, &column)] = get_piece(&character);
                column += 1
            }
            _ => panic!("Invalid character in Fen"),
        }
    }
}

pub fn get_piece(piece: &char) -> Piece {
    let mut lower: bool = true;

    if piece.is_uppercase() {
        lower = false;
    }
    let piece = piece.to_ascii_lowercase();

    let piece_type = match piece {
        'p' => PieceType::Pawn,
        'r' => PieceType::Rook,
        'n' => PieceType::Knight,
        'b' => PieceType::Bishop,
        'q' => PieceType::Queen,
        'k' => PieceType::King,
        _ => panic!("Piece not recognized in Fen"),
    };

    if lower {
        Piece::Black(piece_type)
    } else {
        Piece::White(piece_type)
    }
}

fn get_piece_lists(board: &[Piece]) -> (PieceList, PieceList) {
    let mut white_pieces = PieceList::new();
    let mut black_pieces: PieceList = PieceList::new();

    for (index, piece) in board.iter().enumerate() {
        match piece {
            Piece::White(piece_type) => white_pieces.add_piece(piece_type, &(index as i8)),
            Piece::Black(piece_type) => black_pieces.add_piece(piece_type, &(index as i8)),
            _ => (),
        }
    }

    (white_pieces, black_pieces)
}

fn get_turn(fen: &str) -> game::Color {
    match fen {
        "w" => game::Color::White,
        "b" => game::Color::Black,
        _ => panic!("Invalid color in Fen"),
    }
}

fn get_castling(fen: &str) -> Vec<[bool; 4]> {
    let mut castling: Vec<[bool; 4]> = Vec::new();
    let mut rights = [false; 4];

    for right in fen.chars() {
        match right {
            'K' => rights[0] = true,
            'Q' => rights[1] = true,
            'k' => rights[2] = true,
            'q' => rights[3] = true,
            '-' => (),
            _ => panic!("Invalid character in Fen"),
        }
    }
    castling.push(rights);
    castling
}

fn get_en_passant(fen: &str) -> Vec<Option<i8>> {
    let mut en_passant: Vec<Option<i8>> = Vec::new();
    let mut row: u32 = 0;
    let mut column: u32 = 0;

    for character in fen.chars() {
        match character {
            'A'..='z' => column += letter_to_column(character),
            '0'..='9' => row += character.to_digit(DECIMAL_RADIX).unwrap() - 1,
            '-' => {
                en_passant.push(None);
                return en_passant;
            }
            _ => panic!("Invalid character in Fen"),
        }
    }

    let index = row_column_to_index(&row, &column) as i8;
    en_passant.push(Some(index));
    en_passant
}

fn get_half_move_clock(fen: &str) -> Vec<u16> {
    let mut half_move_clock: Vec<u16> = Vec::new();

    let clock: u16 = match fen.parse() {
        Ok(clock) => clock,
        Err(_) => 0,
    };

    half_move_clock.push(clock);
    half_move_clock
}

fn get_full_move(fen: &str) -> i32 {
    match fen.parse() {
        Ok(move_number) => move_number,
        Err(_) => 0,
    }
}
pub fn letter_to_column(letter: char) -> u32 {
    match letter {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => panic!("Invalid character in Fen"),
    }
}

pub fn row_column_to_index(row: &u32, column: &u32) -> usize {
    (row * ROW_SIZE + MAILBOX_OFF_SET + ROW_OFF_SET + column) as usize
}
