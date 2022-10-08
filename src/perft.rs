use crate::{game, make_move, move_gen, move_notation, unmake};

pub fn perft(depth: i8, mut game: game::GameInfo) -> (game::GameInfo, u64) {
    let mut nodes: u64 = 0;
    let mut temp: u64 = 0;

    if depth == 0 {
        return (game, 1);
    }

    let moves = move_gen::move_gen(&mut game);

    for mut movement in moves {
        game = make_move::make_move(game, &mut movement);
        //game.print_board();

        (game, temp) = perft(depth - 1, game);
        nodes += temp;
        game = unmake::unmake_move(game, movement);
        //game.print_board();
    }

    (game, nodes)
}

pub fn perft_by_nodes(depth: i8, mut game: game::GameInfo) {
    let moves = move_gen::move_gen(&mut game);
    let mut nodes: u64 = 0;
    let mut total = 0;
    for mut movement in moves {
        game = make_move::make_move(game, &mut movement);
        (game, nodes) = perft(depth - 1, game);
        total += nodes;
        println!(
            "{}: {}",
            move_notation::get_move_notation(&movement, game.board),
            nodes
        );
        game = unmake::unmake_move(game, movement);
    }

    println!("total nodes:{}", total)
}
