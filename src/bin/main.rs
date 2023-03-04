use chess::api;
use http_types::headers::HeaderValue;
use std::env;
use tide::security::{CorsMiddleware, Origin};

#[async_std::main]
async fn main() -> tide::Result<()> {
    env::set_var("RUST_BACKTRACE", "0");

    let mut app = tide::new();

    let cors = CorsMiddleware::new()
        .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
        .allow_origin(Origin::from("*"))
        .allow_credentials(false);

    app.with(cors);

    app.at("/getMoves").get(api::get_moves);
    app.at("/makeMove").get(api::make_move);
    app.at("/getBest").get(api::get_best);
    app.listen("127.0.0.1:8080").await?;

    Ok(())
}
