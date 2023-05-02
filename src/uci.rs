use std::{process, io::{Write, Read, BufRead, BufReader}, time::Duration, thread};

use crate::{move_gen, fen_writer, piece, game, fen_reader, model, alpha_beta_search, eval, make_move, api};

#[derive(Debug)]
pub enum WinSide {
    White,
    Black,
    Draw,
}

pub fn get_engine(engine_path: &str) -> process::Child {
    process::Command::new(engine_path)
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("Failed to start engine")
}

pub fn play_game(
    engine_path: &str,
    level: i64,
    color: game::Color,
    net: Option<model::Net>,
    time_limit: Duration,
) -> Result<WinSide, Box<dyn std::error::Error>> {
    let mut engine = get_engine(engine_path);
    let mut game = game::GameInfo::new();
    let mut moves: Vec<String> = Vec::new();

    let mut stdin = engine.stdin.as_mut().expect("Failed to open stdin");
    let mut stdout = engine.stdout.as_mut().expect("Failed to open stdout");
    
    setup_engine(stdin, stdout, level)?;

    if color == game::Color::Black {
        play_engine_turn(stdin, stdout, &mut game, &mut moves, &time_limit)?;
    }

    loop {
        play_player_turn(stdin, &mut game, &mut moves, net.as_ref(), &time_limit)?;

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

        play_engine_turn(stdin, stdout,&mut game, &mut moves, &Duration::from_millis(5))?;

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

fn setup_engine(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout, level: i64) -> Result<(), Box<dyn std::error::Error>> {
    put(stdin, "uci\n");
    get(stdin, stdout).unwrap();
    put(stdin, &format!("setoption name Skill Level value {}\n", level));
    put(stdin, "ucinewgame\n");
    put(stdin, "position startpos\n");

    Ok(())
}

fn play_engine_turn(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout,game: &mut game::GameInfo, moves: &mut Vec<String>, time_limit: &Duration) -> Result<(), Box<dyn std::error::Error>> {
    put(stdin, &format!("go movetime {}\n", time_limit.as_millis()));
    thread::sleep(*time_limit);
    let best_move = get(stdin, stdout).unwrap().split_whitespace().collect::<Vec<&str>>()[1].to_owned();
    
    moves.push(best_move.clone());
    let movements = moves.iter().map(|x| x.to_owned()).collect::<Vec<String>>().join(" ");
    
    put(stdin, &format!("position startpos moves {}\n", movements));
    put(stdin, "d\n");
    let fen = get(stdin, stdout).unwrap(). replace("Fen: ", "");
    fen_reader::read_fen_keep_transposition_table(&fen, game);

    Ok(())
}

fn play_player_turn(stdin: &mut std::process::ChildStdin,game: &mut game::GameInfo, moves: &mut Vec<String>, net: Option<&model::Net>, time_limit: &Duration) -> Result<(), Box<dyn std::error::Error>> {
    let mut best_move = match net {
        Some(net) => alpha_beta_search::iterative_deepening_time_limit_net(game, 100, *time_limit, net).unwrap(),
        None => alpha_beta_search::iterative_deepening_time_limit(game, 100, *time_limit).0.unwrap(),
    };
    make_move::make_move(game, &mut best_move);
    let uci = move_to_uci(best_move);
    moves.push(uci);
    put(stdin, &mut &format!("position startpos moves {}\n", moves.join(" ")));

    Ok(())
}

pub fn get(output: &mut process::ChildStdin, input: &mut process::ChildStdout) -> Result<String, Box<dyn std::error::Error>> {

    let mut buffer = String::new();
    let mut buf_reader = BufReader::new(input);

    loop {
        buffer.clear();
        buf_reader.read_line(&mut buffer)?;
        println!("{}",buffer);
        if buffer.starts_with("bestmove") || buffer.starts_with("Fen:") || buffer.starts_with("uciok") || buffer.starts_with("No such option:") {
            break;
        }
    }

    Ok(buffer)
}



pub fn put<'a>(mut output: &'a  process::ChildStdin, command: &'a str) -> (){
    output.write_all(command.as_bytes()).expect("Failed to write to stdin");
    thread::sleep(Duration::from_millis(25));
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

pub fn uci_to_move(movement: &str, game: &mut game::GameInfo) -> Result<move_gen::Move, Box<dyn std::error::Error>>{

    let origin_letter = fen_reader::letter_to_column(movement.chars().nth(0).unwrap());
    let origin:i8 = (fen_reader::row_column_to_index(&movement.chars().nth(1).unwrap().to_digit(10).unwrap(), &origin_letter) -10).try_into().unwrap();
    
    let destiny_letter = fen_reader::letter_to_column(movement.chars().nth(2).unwrap());
    let destiny:i8 = (fen_reader::row_column_to_index(&movement.chars().nth(3).unwrap().to_digit(10).unwrap(), &destiny_letter)- 10).try_into().unwrap();
    
    for m in move_gen::move_gen(game) {
        if m.origin == origin && m.destiny == destiny {
            return Ok(m);
        }
    }
    Err("Invalid movement".into())
}