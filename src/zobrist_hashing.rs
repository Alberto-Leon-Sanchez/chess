use rand::{self, Rng};
use crate::{piece::{self, PieceType},game::Color,api::board120_to_board64};

pub static mut HASH: Hash = Hash::new();

#[derive(Clone)]
pub struct Hash{

    pub white_pieces:HashingNumbers,
    pub black_piece:HashingNumbers,
    pub turn:u64,
    pub castling:[u64;4],
    pub en_passant:[u64;8],
}

impl Hash{
    
    pub const fn new()->Hash{
        Hash{
            white_pieces:HashingNumbers::new(),
            black_piece:HashingNumbers::new(),
            turn:0,
            castling: [0;4],
            en_passant:[0;8],
        }
    }

    pub fn print(&self){
        println!("turn: {}",self.turn);
        println!("castling: {:?}",self.castling);
        println!("en_passant: {:?}",self.en_passant);
    }

    pub fn randomize(&mut self){
        self.white_pieces.randomize();
        self.black_piece.randomize();
        self.turn = rand::thread_rng().gen();
        for i in 0..4{
            self.castling[i] = rand::thread_rng().gen();
        }
        for i in 0..8{
            self.en_passant[i] = rand::thread_rng().gen();
        }
    }

    pub fn hash_move(&self,piece:PieceType,hash:&mut u64, index:i8,color:Color){

        match color{
            Color::White=>{
                self.white_pieces.hash_move(piece,hash,board120_to_board64(index));
            },
            Color::Black=>{
                self.black_piece.hash_move(piece,hash,board120_to_board64(index));
            }
        }

        *hash ^= self.turn;

    }

    pub fn hash_castling(&self,hash:&mut u64,castling:u8){
        *hash ^= self.castling[castling as usize];
    }

    pub fn hash_en_passant(&self,hash:&mut u64,en_passant:u8){
        *hash ^= self.en_passant[(en_passant%10 - 1) as usize];
    }


    pub fn get_hash(&self,black_pieces:&piece::PieceList,white_pieces:&piece::PieceList,turn:&Color,castling:&[bool;4],en_passant:&Option<i8>)-> u64{
        let mut hash:u64=0;
        
        Self::hash_piece_list(white_pieces, &mut hash, &self.white_pieces);
        Self::hash_piece_list(black_pieces, &mut hash, &self.black_piece);

        if let Color::Black = turn {
            hash ^= self.turn
        }

        for (index,right) in castling.iter().enumerate(){
            if *right{
                hash ^= self.castling[index];
            }
        }

        if let Some(pos) = en_passant{
            hash ^= self.en_passant[(pos%10 - 1) as usize];
        }

        hash
    }

    fn hash_piece_list(pieces: &piece::PieceList, hash: &mut u64, hash_numbers: &HashingNumbers){
        for piece in &pieces.pawns{
            if *hash == 0{
                *hash = hash_numbers.pawns[board120_to_board64(*piece) as usize];
            }else{
                *hash ^= hash_numbers.pawns[board120_to_board64(*piece) as usize];
            }
        }
        
        for piece in &pieces.knights{
            if *hash == 0{
                *hash = hash_numbers.knights[board120_to_board64(*piece) as usize];
            }else{
                *hash ^= hash_numbers.knights[board120_to_board64(*piece) as usize];
            }
        }

        for piece in &pieces.bishops{
            if *hash == 0{
                *hash = hash_numbers.bishops[board120_to_board64(*piece) as usize];
            }else{
                *hash ^= hash_numbers.bishops[board120_to_board64(*piece) as usize];
            }
        }

        for piece in &pieces.rooks{
            if *hash == 0{
                *hash = hash_numbers.rooks[board120_to_board64(*piece) as usize];
            }else{
                *hash ^= hash_numbers.rooks[board120_to_board64(*piece) as usize];
            }
        }

        for piece in &pieces.queens{
            if *hash == 0{
                *hash = hash_numbers.queens[board120_to_board64(*piece) as usize];
            }else{
                *hash ^= hash_numbers.queens[board120_to_board64(*piece) as usize];
            }
        }

        for piece in &pieces.kings{
            if *hash == 0{
                *hash = hash_numbers.kings[board120_to_board64(*piece) as usize];
            }else{
                *hash ^= hash_numbers.kings[board120_to_board64(*piece) as usize];
            }
        }
    }
    
}

#[derive(Clone)]
pub struct HashingNumbers{

    pub pawns:[u64;64],
    pub knights:[u64;64],
    pub bishops:[u64;64],
    pub rooks:[u64;64],
    pub queens:[u64;64],
    pub kings:[u64;64]

}

impl HashingNumbers{
    const fn new()->HashingNumbers{
        HashingNumbers{
            pawns:[0;64],
            knights:[0;64],
            bishops:[0;64],
            rooks:[0;64],
            queens:[0;64],
            kings:[0;64]
        }
    }

    fn randomize(&mut self){
        let mut rng = rand::thread_rng();
        
        for i in 0..64{
            self.pawns[i] = rng.gen();
            self.knights[i] = rng.gen();
            self.bishops[i] = rng.gen();
            self.rooks[i] = rng.gen();
            self.queens[i] = rng.gen();
            self.kings[i] = rng.gen();
        }
    }


    fn hash_move(&self,piece:PieceType,hash:&mut u64,index:i8){
        match piece{
            PieceType::Pawn=>{
                *hash ^= self.pawns[index as usize];
            },
            PieceType::Knight=>{
                *hash ^= self.knights[index as usize];
            },
            PieceType::Bishop=>{
                *hash ^= self.bishops[index as usize];
            },
            PieceType::Rook=>{
                *hash ^= self.rooks[index as usize];
            },
            PieceType::Queen=>{
                *hash ^= self.queens[index as usize];
            },
            PieceType::King=>{
                *hash ^= self.kings[index as usize];
            }
        }
    }
    
}
