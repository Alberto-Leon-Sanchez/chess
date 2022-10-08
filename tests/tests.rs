use chess::{
    fen_positions::get_fen_positions, fen_reader, make_move, move_gen, perft::perft, piece, unmake, zobrist_hashing, game
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

    game = make_move::make_move(game, &mut movement);
    let compare = game.clone();
    movement = move_gen::Move {
        origin: 62,
        destiny: 71,
        destiny_piece: piece::Piece::Black(piece::PieceType::Pawn),
        promotion: None,
    };

    game = make_move::make_move(game, &mut movement);
    game = unmake::unmake_move(game, movement);

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
    game = make_move::make_move(game, &mut movement);

    let mut movement2 = move_gen::Move {
        origin: 95,
        destiny: 93,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    game = make_move::make_move(game, &mut movement2);

    game = unmake::unmake_move(game, movement2);
    game = unmake::unmake_move(game, movement);

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
    game = make_move::make_move(game, &mut movement);

    let mut movement2 = move_gen::Move {
        origin: 95,
        destiny: 97,
        destiny_piece: piece::Piece::Empty,
        promotion: None,
    };
    game = make_move::make_move(game, &mut movement2);

    game = unmake::unmake_move(game, movement2);
    game = unmake::unmake_move(game, movement);

    assert!(game.equal(compare))
}

#[test]
fn initial_position_depth_5() {
    let game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ");
    let mut nodes = 0;

    (_, nodes) = perft(5, game);

    assert_eq!(4865609, nodes)
}

#[test]
fn initial_position_depth_6() {
    let game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ");
    let mut nodes = 0;

    (_, nodes) = perft(6, game);

    assert_eq!(119060324, nodes)
}

#[test]
fn positions() {
    let positions = get_fen_positions();

    for position in positions {
        println!("{}", position.fen);
        let game = fen_reader::read_fen(&position.fen);
        let mut nodes: u64 = 0;
        (_, nodes) = perft(position.depth, game);

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
fn zobrist_hashing(){

    let game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ");

    let hash = zobrist_hashing::Hash::new();

    let mut hash_code = hash.get_hash(&game.black_pieces, &game.white_pieces, &game.turn, game.castling.last().unwrap(), game.en_passant.last().unwrap());
    let compare = hash_code;
    hash.hash_move(piece::PieceType::Pawn, &mut hash_code, 21, game::Color::White);
    hash.hash_move(piece::PieceType::Pawn, &mut hash_code, 21, game::Color::White);


    assert_eq!(hash_code,compare)
}