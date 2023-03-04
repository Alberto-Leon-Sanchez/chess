#[derive(Clone, Debug)]
pub struct PieceList {
    pub pawns: Vec<i8>,
    pub rooks: Vec<i8>,
    pub knights: Vec<i8>,
    pub bishops: Vec<i8>,
    pub queens: Vec<i8>,
    pub kings: Vec<i8>,
}

impl PieceList {
    pub fn new() -> PieceList {
        PieceList {
            pawns: Vec::new(),
            rooks: Vec::new(),
            knights: Vec::new(),
            bishops: Vec::new(),
            queens: Vec::new(),
            kings: Vec::new(),
        }
    }

    pub fn add_piece(&mut self, piece_type: &PieceType, index: &i8) {
        match piece_type {
            PieceType::Pawn => self.pawns.push(*index),
            PieceType::Rook => self.rooks.push(*index),
            PieceType::Knight => self.knights.push(*index),
            PieceType::Bishop => self.bishops.push(*index),
            PieceType::Queen => self.queens.push(*index),
            PieceType::King => self.kings.push(*index),
        }
    }

    pub fn make_move(&mut self, piece_type: &PieceType, destiny: i8, origin: i8) {
        match piece_type {
            PieceType::Pawn => {
                for (i, x) in self.pawns.iter().enumerate() {
                    if *x == origin {
                        self.pawns[i] = destiny;
                        break;
                    }
                }
            }
            PieceType::Rook => {
                for (i, x) in self.rooks.iter().enumerate() {
                    if *x == origin {
                        self.rooks[i] = destiny;
                        break;
                    }
                }
            }
            PieceType::Knight => {
                for (i, x) in self.knights.iter().enumerate() {
                    if *x == origin {
                        self.knights[i] = destiny;
                        break;
                    }
                }
            }
            PieceType::Bishop => {
                for (i, x) in self.bishops.iter().enumerate() {
                    if *x == origin {
                        self.bishops[i] = destiny;
                        break;
                    }
                }
            }
            PieceType::Queen => {
                for (i, x) in self.queens.iter().enumerate() {
                    if *x == origin {
                        self.queens[i] = destiny;
                        break;
                    }
                }
            }
            PieceType::King => {
                for (i, x) in self.kings.iter().enumerate() {
                    if *x == origin {
                        self.kings[i] = destiny;
                        break;
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, piece_type: &PieceType, index: i8) {
        match piece_type {
            PieceType::Pawn => {
                for (i, x) in self.pawns.iter().enumerate() {
                    if *x == index {
                        self.pawns.remove(i);
                        break;
                    }
                }
            }
            PieceType::Rook => {
                for (i, x) in self.rooks.iter().enumerate() {
                    if *x == index {
                        self.rooks.remove(i);
                        break;
                    }
                }
            }
            PieceType::Knight => {
                for (i, x) in self.knights.iter().enumerate() {
                    if *x == index {
                        self.knights.remove(i);
                        break;
                    }
                }
            }
            PieceType::Bishop => {
                for (i, x) in self.bishops.iter().enumerate() {
                    if *x == index {
                        self.bishops.remove(i);
                        break;
                    }
                }
            }
            PieceType::Queen => {
                for (i, x) in self.queens.iter().enumerate() {
                    if *x == index {
                        self.queens.remove(i);
                        break;
                    }
                }
            }
            PieceType::King => {
                for (i, x) in self.kings.iter().enumerate() {
                    if *x == index {
                        self.kings.remove(i);
                        break;
                    }
                }
            }
        }
    }

    pub fn diference(&mut self, piece_list: &PieceList) {
        for x in &piece_list.pawns {
            self.remove(&PieceType::Pawn, *x);
        }
        for x in &piece_list.rooks {
            self.remove(&PieceType::Rook, *x);
        }
        for x in &piece_list.knights {
            self.remove(&PieceType::Knight, *x);
        }
        for x in &piece_list.bishops {
            self.remove(&PieceType::Bishop, *x);
        }
        for x in &piece_list.queens {
            self.remove(&PieceType::Queen, *x);
        }
        for x in &piece_list.kings {
            self.remove(&PieceType::King, *x);
        }
    }

    pub fn add(&mut self, piece_list: PieceList) {
        for x in piece_list.pawns {
            self.add_piece(&PieceType::Pawn, &x);
        }
        for x in piece_list.rooks {
            self.add_piece(&PieceType::Rook, &x);
        }
        for x in piece_list.knights {
            self.add_piece(&PieceType::Knight, &x);
        }
        for x in piece_list.bishops {
            self.add_piece(&PieceType::Bishop, &x);
        }
        for x in piece_list.queens {
            self.add_piece(&PieceType::Queen, &x);
        }
        for x in piece_list.kings {
            self.add_piece(&PieceType::King, &x);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl PieceType {
    pub fn is_sliding(&self) -> bool {
        match self {
            PieceType::Rook => true,
            PieceType::Bishop => true,
            PieceType::Queen => true,
            _ => false,
        }
    }

    pub fn direction(&self, direction: i8) -> bool {
        let direction = direction.abs();

        match self {
            PieceType::Rook => direction == 1 || direction == 10,
            PieceType::Bishop => direction == 9 || direction == 11,
            PieceType::Queen => {
                direction == 1 || direction == 10 || direction == 9 || direction == 11
            }
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Piece {
    White(PieceType),
    Black(PieceType),
    Empty,
    Outside,
}
