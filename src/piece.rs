pub struct PieceList{

    pub pawns:Vec<i8>,
    pub rooks:Vec<i8>,
    pub knights:Vec<i8>,
    pub bishops:Vec<i8>,
    pub queens:Vec<i8>,
    pub kings:Vec<i8>

}

impl PieceList{
    pub fn new() -> PieceList{
        PieceList{
            pawns:Vec::new(),
            rooks:Vec::new(),
            knights:Vec::new(),
            bishops:Vec::new(),
            queens:Vec::new(),
            kings:Vec::new()
        }
    }
    
    pub fn add_piece(&mut self,piece_type:&PieceType,index: &i8){

        match piece_type{
            PieceType::Pawn => self.pawns.push(*index),
            PieceType::Rook => self.rooks.push(*index),
            PieceType::Knight => self.knights.push(*index),
            PieceType::Bishop => self.bishops.push(*index),
            PieceType::Queen => self.queens.push(*index),
            PieceType::King => self.kings.push(*index)
        }
        
    }

}

#[derive(Copy, Clone,Debug)]
pub enum PieceType{
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

    
#[derive(Copy, Clone,Debug)]
pub enum Piece{
    White(PieceType),
    Black(PieceType),
    Empty,
    Outside
}


