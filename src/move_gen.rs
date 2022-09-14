use crate::piece;
use crate::game;
use crate::attack_gen;

pub const DIAGONAL_SLIDING:[i8;4] = [9,11,-11,-9];
pub const LATERAL_SLIDING:[i8;4] = [10,1,-10,-1];
pub const KNIGHT_SLIDING:[i8;8] = [-21, -19,-12, -8, 8, 12, 19, 21];

pub struct Move{
    pub origin:i8,
    pub destiny:i8,
    pub destiny_piece: piece::Piece,
}

pub fn move_gen(game:&mut game::GameInfo) -> Vec<Move>{

    let mut moves:Vec<Move> = Vec::new();
    let attacks = attack_gen::attack_gen(game);

    let mut piece_list = &game.white_pieces;

    if matches!(game.turn, game::Color::Black){
        piece_list = &game.black_pieces;
    }

    moves.append(&mut direction_sliding(&piece_list.bishops, &game.board, &game.turn,&DIAGONAL_SLIDING));
    moves.append(&mut direction_sliding(&piece_list.queens, &game.board, &game.turn,&DIAGONAL_SLIDING));

    moves.append(&mut direction_sliding(&piece_list.rooks, &game.board, &game.turn,&LATERAL_SLIDING));
    moves.append(&mut direction_sliding(&piece_list.queens, &game.board, &game.turn,&LATERAL_SLIDING));

    moves.append(&mut knight_moves(&piece_list.knights, &game.board, &game.turn));

    moves.append(&mut pawn_moves(&piece_list.pawns, &game.board, &game.turn, game.en_passant.last().unwrap()));
    moves.append(&mut king_moves(&piece_list.kings, &game.board, &game.turn, game.castling.last().unwrap(), &attacks));
    
    moves
}   

fn direction_sliding(piece_list:&Vec<i8>,board:&[piece::Piece;120],turn: &game::Color,direction:&[i8]) -> Vec<Move>{
    let mut moves:Vec<Move> = Vec::new();

    for piece in piece_list{
        for direction in direction{
            let mut destiny = *piece + direction;
            let mut destiny_piece = board[destiny as usize];
            
            loop{
                match destiny_piece{
                    piece::Piece::White(_) => {
                        if matches!(turn, game::Color::Black){
                            moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                        }
                        break;
                    }
                    piece::Piece::Black(_) => {
                        if matches!(*turn,game::Color::White) {
                            moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                        }
                        break;
                    },
                    piece::Piece::Empty => {
                        moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:piece::Piece::Empty});
                        destiny += direction;
                        destiny_piece = board[destiny as usize];

                    },
                    piece::Piece::Outside => break,
                }
            }

        }
    }
    moves
}

fn knight_moves(piece_list:&Vec<i8>,board:&[piece::Piece;120],turn: &game::Color) -> Vec<Move>{
    let mut moves:Vec<Move> = Vec::new();

    for piece in piece_list{
        for direction in KNIGHT_SLIDING{
            let destiny = *piece + direction;
            let destiny_piece = board[destiny as usize];
             
            match destiny_piece{
                piece::Piece::White(_) => {
                    if matches!(turn, game::Color::Black){
                        moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                    }
                },
                piece::Piece::Black(_) => {
                    if matches!(*turn,game::Color::White) {
                        moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                    }
                },
                piece::Piece::Empty => {
                    moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:piece::Piece::Empty});
                },
                piece::Piece::Outside => (),
            }
        }
    }
    moves
}

fn pawn_moves(piece_list:&Vec<i8>,board:&[piece::Piece;120],turn: &game::Color, en_passant:&Option<i8>) -> Vec<Move>{
    let mut moves:Vec<Move> = Vec::new();

    let mut movement = 10;

    if matches!(turn,game::Color::Black){
        movement = -10;
    }

    for piece in piece_list{
        let mut destiny = *piece + movement;
        let mut destiny_piece = board[destiny as usize];
        match destiny_piece{
            piece::Piece::Empty => {
                moves.push(Move{origin:*piece, destiny, destiny_piece});
                
                if !is_pawn_move(piece, turn){
                    destiny = *piece + movement*2;
                    destiny_piece = board[destiny as usize];
                    match destiny_piece{
                        piece::Piece::Empty => {
                            moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                        }
                        _ => ()
                    }
                }
            }
            _ => {},
        }

        destiny = *piece + movement - 1;
        destiny_piece = board[destiny as usize];
        match destiny_piece{
            piece::Piece::Black(_) => {
                if matches!(*turn,game::Color::White) {
                    moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                }
            },
            piece::Piece::White(_) => {
                if matches!(*turn,game::Color::Black) {
                    moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                }
            }
            _ => {
                match en_passant{
                    Some(pos) => {
                        if destiny == *pos{
                            moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                        }
                    },
                    None => (),
                }

            }
        }

        destiny = *piece + movement + 1;
        destiny_piece = board[destiny as usize];
        match destiny_piece{
            piece::Piece::Black(_) => {
                if matches!(*turn,game::Color::White) {
                    moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                }
            },
            piece::Piece::White(_) => {
                if matches!(*turn,game::Color::Black) {
                    moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                }
            }
            _ => {
                match en_passant{
                    Some(pos) => {
                        if destiny == *pos{
                            moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                        }
                    },
                    None => (),
                }
            }
        }

    }

    moves
}

fn is_pawn_move(pos:&i8, turn:&game::Color) -> bool{
    
    if matches!(*turn,game::Color::Black){
        if *pos >= 82 && *pos <= 90 {
            return false;
        }
    }else{
        if *pos >= 31 && *pos <= 39 {
            return false;
        }
    }
    true
}


fn king_moves(piece_list:&Vec<i8>,board:&[piece::Piece;120],turn: &game::Color, castling:&[bool;4], attacks:&[u8;120]) -> Vec<Move>{
    let mut moves:Vec<Move> = Vec::new();

    for piece in piece_list{
        for direction in DIAGONAL_SLIDING.iter().chain(LATERAL_SLIDING.iter()){
            let destiny = *piece + direction;
            let destiny_piece = board[destiny as usize];

            if attacks[destiny as usize] > 0{
                continue;
            }

            match destiny_piece{
                piece::Piece::White(_) => {
                    if matches!(turn, game::Color::Black){
                        moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                    }
                },
                piece::Piece::Black(_) => {
                    if matches!(*turn,game::Color::White) {
                        moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:destiny_piece});
                    }
                },
                piece::Piece::Empty => {
                    moves.push(Move{origin:*piece, destiny:destiny, destiny_piece:piece::Piece::Empty});
                }
                piece::Piece::Outside => {},
            }
        }

        match turn {
            game::Color::White => {
                if castling[0] && is_empty(*piece+1, board) && is_empty(*piece+2, board){
                    moves.push(Move{origin:*piece, destiny:*piece + 2, destiny_piece: piece::Piece::Empty })
                }
                if castling[1] && is_empty(*piece-1, board) && is_empty(*piece-2, board) && is_empty(*piece-3, board){
                    moves.push(Move{origin:*piece, destiny:*piece - 2, destiny_piece: piece::Piece::Empty })
                }
            },
            game::Color::Black => {
                if castling[2] && is_empty(*piece+1, board) && is_empty(*piece+2, board){
                    moves.push(Move{origin:*piece, destiny:*piece + 2, destiny_piece: piece::Piece::Empty })
                }
                if castling[3] && is_empty(*piece-1, board) && is_empty(*piece-2, board) && is_empty(*piece-3, board){
                    moves.push(Move{origin:*piece, destiny:*piece - 2, destiny_piece: piece::Piece::Empty })
                }
            }
        }
    }

    

    moves
}

fn is_empty(pos:i8,board:&[piece::Piece;120]) -> bool{

    if matches!(board[pos as usize],piece::Piece::Empty){
        return true;
    }
    else{ 
        return false;
    }
}
