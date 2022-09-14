use crate::piece;
use crate::fen_reader;

const BLACK_KING: char = '\u{2654}';
const BLACK_QUEEN: char = '\u{2655}';
const BLACK_ROOK: char = '\u{2656}';
const BLACK_BISHOP: char = '\u{2657}';
const BLACK_KNIGHT: char = '\u{2658}';
const BLACK_PAWN: char = '\u{2659}';

const WHITE_KING: char = '\u{265A}';
const WHITE_QUEEN: char = '\u{265B}';
const WHITE_ROOK: char = '\u{265C}';
const WHITE_BISHOP: char = '\u{265D}';
const WHITE_KNIGHT: char = '\u{265E}';
const WHITE_PAWN: char = '\u{265F}';

const EMPTY: char = '\u{25A1}';

pub enum Color{
    White,
    Black
}

impl Color{
    pub fn change_turn(&self)-> Color{
        match self{
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

pub struct GameInfo{

    pub board:[piece::Piece;120],
    pub white_pieces: piece::PieceList,
    pub black_pieces: piece::PieceList,
    pub turn: Color,
    pub castling: Vec<[bool;4]>,
    pub en_passant: Vec<Option<i8>>,
    pub half_move_clock: Vec<u8>,
    pub full_move: i32
}

impl GameInfo{

    pub fn print_board(&self){
        for i in (0..=7).rev(){
            for j in 0..=7{
                
                let index = fen_reader::row_column_to_index(&i,&j);
                
                match self.board[index as usize]{
                    piece::Piece::Empty => print!("{}",EMPTY),
                    piece::Piece::White(piece::PieceType::Pawn) => print!("{}",WHITE_PAWN),
                    piece::Piece::White(piece::PieceType::Rook) => print!("{}",WHITE_ROOK),
                    piece::Piece::White(piece::PieceType::Knight) => print!("{}",WHITE_KNIGHT),
                    piece::Piece::White(piece::PieceType::Bishop) => print!("{}",WHITE_BISHOP),
                    piece::Piece::White(piece::PieceType::Queen) => print!("{}",WHITE_QUEEN),
                    piece::Piece::White(piece::PieceType::King) => print!("{}",WHITE_KING),
                    piece::Piece::Black(piece::PieceType::Pawn) => print!("{}",BLACK_PAWN),
                    piece::Piece::Black(piece::PieceType::Rook) => print!("{}",BLACK_ROOK),
                    piece::Piece::Black(piece::PieceType::Knight) => print!("{}",BLACK_KNIGHT),
                    piece::Piece::Black(piece::PieceType::Bishop) => print!("{}",BLACK_BISHOP),
                    piece::Piece::Black(piece::PieceType::Queen) => print!("{}",BLACK_QUEEN),
                    piece::Piece::Black(piece::PieceType::King) => print!("{}",BLACK_KING),
                    piece::Piece::Outside => ()
                }
            }
            println!();
        }
    }
}