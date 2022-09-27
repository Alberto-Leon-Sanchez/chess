mod game;
mod piece; 
mod fen_reader;
mod move_gen;
mod api;
mod make_move;
pub mod fen_writer;
mod attack_gen;
mod unmake;
mod move_notation;
mod fen_positions;
mod tests;

use http_types::headers::HeaderValue;
use move_gen::move_gen;
use tide::security::{CorsMiddleware, Origin};
use std::env;

#[async_std::main]
async fn main() -> tide::Result<()>{
    
    env::set_var("RUST_BACKTRACE", "0");
    let mut game = fen_reader::read_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

    let mut result = 0;
    (game,result) = perft(6, game);
    //perft_by_nodes(2, game);
    println!("{}",result);

    let mut app = tide::new();

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);

    app.at("/getMoves").get(|request|  api::get_moves(request));
    app.at("/makeMove").get( |request| api::make_move(request));
    app.listen("127.0.0.1:8080").await?;
       
    Ok(())
}

fn perft(depth:i8,mut game:game::GameInfo) -> (game::GameInfo,u64){

    let mut nodes:u64 = 0;
    let mut temp:u64 = 0;

    if depth == 0{
        return (game,1);
    }

    let moves= move_gen(&mut game);

    for mut movement in moves{
        game = make_move::make_move(game, &mut movement);
        //game.print_board();
      
        (game,temp) = perft(depth-1, game);
        nodes += temp;
        game = unmake::unmake_move(game, movement);
        //game.print_board();

    }

    (game,nodes)
}


fn perft_by_nodes(depth:i8,mut game:game::GameInfo){

    let moves = move_gen(&mut game);
    let mut nodes:u64 = 0;
    let mut total = 0;
    for mut movement in moves{
        
        game = make_move::make_move(game, &mut movement);
        (game,nodes) = perft(depth-1, game);
        total += nodes;
        println!("{}: {}",move_notation::get_move_notation(&movement,game.board),nodes);
        game = unmake::unmake_move(game, movement);

    }
    
    println!("total nodes:{}",total)
}