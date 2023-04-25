use std::{process, io::{Write, Read}};

use crate::{move_gen, fen_writer, piece, game, fen_reader, model, alpha_beta_search, eval, make_move};

pub enum WinSide {
    White,
    Black,
    Draw,
}

fn get_engine(engine_path: &str) -> process::Child {
    process::Command::new(engine_path)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("Failed to start engine")
}

pub fn play_game(engine_path: &str, elo: i64, color: game::Color, net: Option<model::Net>) -> Result<WinSide, Box<dyn std::error::Error>> {
    let mut engine = get_engine(engine_path);
    let mut game = game::GameInfo::new();
    let mut moves: Vec<String> = Vec::new();
    let regex_fen = regex::Regex::new(r"(?<=FEN:\s)[^ ]+ .*").unwrap();

    let stdin = engine.stdin.as_mut().expect("Failed to open stdin");
    let stdout = engine.stdout.as_mut().expect("Failed to open stdout");

    setup_engine(stdin, stdout, elo)?;

    if color == game::Color::Black {
        play_engine_turn(stdin, stdout, &regex_fen, &mut game, &mut moves)?;
    }

    loop {
        
        play_player_turn(stdin, &mut game, &mut moves, net.as_ref())?;
        
        match eval::is_game_over(&mut game) {
            Some(value) => {
                if value == 1.0 {
                    return Ok(WinSide::White);
                } else if value == -1.0 {
                    return Ok(WinSide::Black);
                } else {
                    return Ok(WinSide::Draw);
                }
            }
            None => (),
        }

        play_engine_turn(stdin, stdout, &regex_fen, &mut game, &mut moves)?;

        match eval::is_game_over(&mut game) {
            Some(value) => {
                if value == 1.0 {
                    return Ok(WinSide::White);
                } else if value == -1.0 {
                    return Ok(WinSide::Black);
                } else {
                    return Ok(WinSide::Draw);
                }
            }
            None => (),
        }
    }
}

fn setup_engine(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout, elo: i64) -> Result<(), Box<dyn std::error::Error>> {
    get(stdin, stdout)?;
    put(stdin, "uci\n");
    get(stdin, stdout)?;
    put(stdin, &format!("setoption name Skill Level value {}\n", elo));
    get(stdin, stdout)?;
    put(stdin, "ucinewgame\n");
    get(stdin, stdout)?;
    put(stdin, "position startpos\n");
    get(stdin, stdout)?;

    Ok(())
}

fn play_engine_turn(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout, regex_fen: &regex::Regex, game: &mut game::GameInfo, moves: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    put(stdin, "go movetime 500\n");
    let best_move = get(stdin, stdout)?.split_whitespace().last().unwrap().to_owned();
    moves.push(best_move.clone());
    put(stdin, &format!("position startpos moves {}\n", best_move));
    put(stdin, "d\n");
    let fen = get(stdin, stdout).unwrap();
    let fen = regex_fen.captures(&fen).unwrap().get(0).unwrap().as_str();
    fen_reader::read_fen_keep_transposition_table(fen, game);

    Ok(())
}

fn play_player_turn(stdin: &mut std::process::ChildStdin,game: &mut game::GameInfo, moves: &mut Vec<String>, net: Option<&model::Net>) -> Result<(), Box<dyn std::error::Error>> {
    let mut best_move = match net {
        Some(net) => alpha_beta_search::best_move_net(3, game, net),
        None => alpha_beta_search::best_move(5, game),
    };
    make_move::make_move(game, &mut best_move);
    let uci = move_to_uci(best_move);
    moves.push(uci);
    put(stdin, &mut &format!("position startpos moves {}\n", moves.join(" ")));

    Ok(())
}

fn get(output: &mut process::ChildStdin, input: &mut process::ChildStdout) -> Result<String, Box<dyn std::error::Error>> {
    output.write_all("isready\n".as_bytes())?;
    println!("engine:\n");
    let mut buffer = String::new();
    loop {
        input.read_to_string(&mut buffer)?;
        if buffer.ends_with("readyok") {
            break;
        }
        println!("{}", buffer);
    }
    Ok(buffer)
}

fn put<'a>(mut output: &'a  process::ChildStdin, command: &'a str) -> (){
    println!("you:\n");
    output.write_all(command.as_bytes()).expect("Failed to write to stdin");
}

fn move_to_uci(movement: move_gen::Move) -> String{
    
    let mut uci = String::new();

    uci.push_str(&fen_writer::index_to_letter_pos(&movement.origin));
    uci.push_str(&fen_writer::index_to_letter_pos(&movement.destiny));

    match movement.promotion {
        Some(piece) => {
            match piece {
                piece::PieceType::Queen => uci.push('q'),
                piece::PieceType::Rook => uci.push('r'),
                piece::PieceType::Bishop => uci.push('b'),
                piece::PieceType::Knight => uci.push('n'),
                _ => panic!("Invalid promotion"),
            }
        }
        None => (),
    }

    uci
}