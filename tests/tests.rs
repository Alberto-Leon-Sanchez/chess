use chess::{
    fen_positions::get_fen_positions, fen_reader, make_move, move_gen::{self, move_gen}, perft::perft, piece, unmake, zobrist_hashing::HASH, game, alpha_beta_search
};

#[cfg(test)]
#[test]
fn en_passant() {
    let mut game =
        fen_reader::read_fen("rnbqkbnr/pppppppp/8/1P6/8/8/P1PPPPPP/RNBQKBNR b KQkq - 0 1");
    let mut movement = move_gen::Move {
        origin: 81,
        destiny: 61,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };

    make_move::make_move(&mut game,&mut movement);
    let compare = game.clone();
    movement = move_gen::Move {
        origin: 62,
        destiny: 71,
        destiny_piece: piece::Piece::Black(piece::PieceType::Pawn),
        promotion: None,
    };

    make_move::make_move(&mut game, &mut movement);
    unmake::unmake_move(&mut game, movement);

    assert!(game.equal(compare))
}

#[test]
fn castling_queen_side() {
    let mut game = fen_reader::read_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
    let compare = game.clone();

    let mut movement = move_gen::Move {
        origin: 25,
        destiny: 23,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement);

    let mut movement2 = move_gen::Move {
        origin: 95,
        destiny: 93,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement2);

    unmake::unmake_move(&mut game, movement2);
    unmake::unmake_move(&mut game, movement);

    assert!(game.equal(compare))
}

#[test]
fn castling_king_side() {
    let mut game = fen_reader::read_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
    let compare = game.clone();

    let mut movement = move_gen::Move {
        origin: 25,
        destiny: 27,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement);

    let mut movement2 = move_gen::Move {
        origin: 95,
        destiny: 97,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement2);

    unmake::unmake_move(&mut game, movement2);
    unmake::unmake_move(&mut game, movement);

    assert!(game.equal(compare))
}

#[test]
fn initial_position_depth_5() {
    
    unsafe{
        HASH.randomize();
    }
    
    let mut game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ");
    let mut nodes = 0;

    nodes = perft(5, &mut game);

    assert_eq!(4865609, nodes)
}
#[ignore]
#[test]
fn initial_position_depth_6() {
    
    unsafe{
        HASH.randomize();
    }
    
    let mut game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ");
    let mut nodes = 0;

    nodes = perft(6, &mut game);

    assert_eq!(119060324, nodes)
}

#[test]
fn positions() {
    let positions = get_fen_positions();

    unsafe{
        HASH.randomize();
    }

    for position in positions {
        println!("{}", position.fen);
        let mut game = fen_reader::read_fen(&position.fen);
        let mut nodes: u64 = 0;
        nodes = perft(position.depth, &mut game);

        if nodes != position.nodes {
            assert!(
                false,
                "fen:{} depth:{} nodes:{} nodes calculated:{}",
                position.fen, position.depth, position.nodes, nodes
            )
        }
    }

    assert!(true)
}

#[test]
fn zobrist_hashing_regular(){

    unsafe{
        HASH.randomize();
    }

    let mut game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ");

    let compare = game.hash;
    let mut movement = move_gen::Move {
        origin: 81,
        destiny: 71,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };

    make_move::make_move(&mut game, &mut movement);
    unmake::unmake_move(&mut game, movement);

    assert_eq!(compare,game.hash)
}


#[test]
fn zobrist_hashing_en_passant(){

    unsafe{
        HASH.randomize();
    }

    let mut game =
        fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut movement = move_gen::Move {
        origin: 31,
        destiny: 51,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };

    let compare = game.hash;
    make_move::make_move(&mut game, &mut movement);

    unmake::unmake_move(&mut game, movement);

    assert_eq!(compare,game.hash)

}

#[test]
fn zobrist_hashing_en_passant_capture(){
    unsafe{
        HASH.randomize();
    }

    let mut game = fen_reader::read_fen("rnbqkbnr/ppppppp1/7p/P7/8/8/1PPPPPPP/RNBQKBNR b KQkq - 0 2");

    let mut movement = move_gen::Move{
        origin: 82,
        destiny: 62,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };

    make_move::make_move(&mut game, &mut movement);
    let compare = game.hash;

    movement = move_gen::Move{
        origin:61,
        destiny:72,
        destiny_piece: piece::Piece::Black(piece::PieceType::Pawn),
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement);
    unmake::unmake_move(&mut game, movement);

    assert_eq!(game.hash,compare)

}

#[test]
fn zobrist_hashing_castling_king_side(){
    
    unsafe{
        HASH.randomize();
    }

    let mut game = fen_reader::read_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
    let compare = game.hash;

    let mut movement = move_gen::Move {
        origin: 25,
        destiny: 23,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement);

    let mut movement2 = move_gen::Move {
        origin: 95,
        destiny: 93,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement2);

    unmake::unmake_move(&mut game, movement2);
    unmake::unmake_move(&mut game, movement);

    assert_eq!(game.hash,compare)

}

#[test]
fn zobrist_hashing_castling_queen_side(){
    unsafe{
        HASH.randomize();
    }

    let mut game = fen_reader::read_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
    let compare = game.hash;

    let mut movement = move_gen::Move {
        origin: 25,
        destiny: 27,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement);

    let mut movement2 = move_gen::Move {
        origin: 95,
        destiny: 97,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    make_move::make_move(&mut game, &mut movement2);

    unmake::unmake_move(&mut game, movement2);
     unmake::unmake_move(&mut game, movement);

    assert_eq!(game.hash,compare)
}

#[test]
fn zobrist_hashing_capture(){
    unsafe{
        HASH.randomize();
    }

    let mut game = fen_reader::read_fen("rnbqkbnr/ppppppp1/P7/8/7p/8/1PPPPPPP/RNBQKBNR w KQkq - 0 4");

    let mut movement  = move_gen::Move{
        origin:71,
        destiny:82,
        destiny_piece: piece::Piece::Black(piece::PieceType::Pawn),
        promotion: None,
    };

    let compare = game.hash;
    make_move::make_move(&mut game, &mut movement);

    unmake::unmake_move(&mut game, movement);

    assert_eq!(game.hash,compare)

}

#[test]
fn zobrist_hashing_promotion(){
    unsafe{
        HASH.randomize();
    }

    let mut game = fen_reader::read_fen("rnbqkbnr/pPppppp1/8/8/8/7p/1PPPPPPP/RNBQKBNR w KQkq - 0 5");

    let mut movement  = move_gen::Move{
        origin:82,
        destiny:91,
        destiny_piece: piece::Piece::Black(piece::PieceType::Pawn),
        promotion: Some(piece::PieceType::Queen),
    };

    let compare = game.hash;
    make_move::make_move(&mut game, &mut movement);

    unmake::unmake_move(&mut game, movement);

    assert_eq!(game.hash,compare)
}

#[test]
fn zobrist_hashing(){

    unsafe{
        HASH.randomize();
    }

    let positions = get_fen_positions();

    for position in positions {
        println!("{}", position.fen);
        let mut game = fen_reader::read_fen(&position.fen);
        
        let moves = move_gen::move_gen(&mut game);

        for mut movement in moves{

            let hash = game.hash;
            game.print_board();
            make_move::make_move(&mut game, &mut movement);
            unmake::unmake_move(&mut game, movement);
            game.print_board();
            println!("{},{}",hash,game.hash);
            assert_eq!(hash,game.hash)
        }

    }



}