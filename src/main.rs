mod game;
mod piece; 
mod fen_reader;
mod move_gen;
mod api;
mod make_move;
mod fen_writer;
mod attack_gen;

use http_types::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};


#[async_std::main]
async fn main() -> tide::Result<()>{
    
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

