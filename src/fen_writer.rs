use crate::game;
use crate::fen_reader;
use crate::piece;


pub fn write_fen(gameInfo: &game::GameInfo)-> String{

    let mut fen = String::new();
    let mut count= 0;

    for i in (0..8).rev(){
        for j in 0..8{
            let index = fen_reader::row_column_to_index(&i,&j);

            match gameInfo.board[index]{
                crate::piece::Piece::White(piece) => {
                    if count > 0{
                        fen.push_str(&count.to_string());
                        count = 0;
                    }
                    fen = piece_to_letter(fen, &piece, true);
                },
                crate::piece::Piece::Black(piece) => {
                    if count > 0{
                        fen.push_str(&count.to_string());
                        count = 0;
                    }
                    fen = piece_to_letter(fen, &piece, false);
                },
                crate::piece::Piece::Empty => count += 1,
                crate::piece::Piece::Outside => (),
            }

        }
        if count > 0{
            fen.push_str(&count.to_string());
            count = 0;
        }
        if i>0{
            fen.push('/');
        }else {
            fen.push(' ');
        }
    }

    fen.push(get_turn(gameInfo));
    fen.push(' ');
    
    fen.push_str(&get_castling(gameInfo));
    fen.push(' ');

    fen.push_str(&get_enpassent(gameInfo));
    fen.push(' ');

    fen.push_str(&half_move_clock(gameInfo));
    fen.push(' ');

    fen.push_str(&&full_move_number(gameInfo));


    fen
}

fn piece_to_letter(mut fen:String,piece: &piece::PieceType,white:bool) -> String {

    let mut piece_letter:char;

    match piece {
        piece::PieceType::Pawn => piece_letter = 'P',
        piece::PieceType::Knight => piece_letter = 'N',
        piece::PieceType::Bishop => piece_letter = 'B',
        piece::PieceType::Rook => piece_letter = 'R',
        piece::PieceType::Queen => piece_letter = 'Q',
        piece::PieceType::King => piece_letter = 'K',
    }

    if !white{
        piece_letter = piece_letter.to_ascii_lowercase();
    }
    fen.push(piece_letter);
    fen
}

fn get_turn(gameInfo: &game::GameInfo) -> char{
    
    match gameInfo.turn{
        game::Color::White => 'w',
        game::Color::Black => 'b',
    }
}

fn get_castling(gameInfo:&game::GameInfo) -> String{

    let mut castling = String::new();
    let rights = gameInfo.castling.last().unwrap();

    if rights[0]{
        castling.push('K');
    }
    if rights[1]{
        castling.push('Q');
    }
    if rights[2]{
        castling.push('k');
    }
    if rights[3]{
        castling.push('q');
    }

    if castling == ""{
        castling = "-".to_string();
    }
    castling
}

fn get_enpassent(gameInfo:&game::GameInfo) -> String{
    let mut enpassent = String::new();

    if let Some(square) = gameInfo.en_passant.last().unwrap(){
        enpassent = index_to_letter_pos(square);
    }else{
        enpassent = "-".to_string();
    }
    enpassent
}

fn index_to_letter_pos(index:&i8) -> String{
    let mut letter_pos = String::new();
    let row = index/10 - 1;
    let column = index%10 - 1;
    
    letter_pos.push((column+97) as u8 as char);
    letter_pos.push((row+48) as u8 as char);

    letter_pos
}

fn half_move_clock(gameInfo:&game::GameInfo) -> String{
    let mut half_move = String::new();

    half_move = gameInfo.half_move_clock.last().unwrap().to_string();

    half_move
}

fn full_move_number(gameInfo:&game::GameInfo) -> String{
    let mut full_move = String::new();

    full_move = gameInfo.full_move.to_string();

    full_move
}
