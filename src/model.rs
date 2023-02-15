use std::{vec, io::BufRead};

use tch::{nn::{self, OptimizerConfig, Module}, Tensor};
use crate::{game::{self, GameInfo}, piece::{PieceList, Piece}, move_gen, make_move, unmake::{self, unmake_move}, fen_reader};
use crate::api::{board120_to_board64,board64_to_board120};
use crate::attack_gen;

const N_STEPS:i64 = 14;
const N_EPOCHS:i64 = 10;
const N_GAMES:i64 = 10;
const LAMBDA:f64 = 0.8;

#[derive(Debug)]
struct Net{
    piece_pos: tch::nn::Linear,
    game_state: tch::nn::Linear,
    attacks: tch::nn::Linear,
    hidden: tch::nn::Linear,
}

impl Module for Net{
    fn forward(&self, xs: &tch::Tensor) -> tch::Tensor {

        let piece_pos = self.piece_pos.forward(&xs.slice(0, 0, 384, 1)).relu();
        let game_state = self.game_state.forward(&xs.slice(0, 384, 453, 1)).relu();
        let attacks = self.attacks.forward(&xs.slice(0, 453, 581, 1)).relu();
        let mut result = Tensor::cat(&[piece_pos, game_state, attacks], 0);
      
        result = self.hidden.forward(&result).atanh();

        result
    }
}


fn get_training_games() -> Vec<game::GameInfo>{
    let mut games = vec![];

    let file = std::fs::File::open("/home/castor_cabron/proyectos/chess/games.txt").unwrap();
    let mut lines = std::io::BufReader::new(file).lines();
    
    for i in 0..N_GAMES{
        games.push(fen_reader::read_fen(&lines.nth(i as usize).unwrap().unwrap()));
    }

    games
}

fn pre_proccess(game: &mut game::GameInfo ) -> tch::Tensor{
    
    let bitmaps = piece_lists_to_bitmaps(&game.white_pieces, &game.black_pieces).totype(tch::Kind::Float);
    let game_state = game_state(game).totype(tch::Kind::Float);
    let attacks = attacks(game).totype(tch::Kind::Float);


    Tensor::cat(&[bitmaps, game_state, attacks], 0).totype(tch::Kind::Float)
}

fn piece_lists_to_bitmaps(white: &PieceList, black: &PieceList) -> tch::Tensor{
    
    let to_bitmap = |white: &Vec<i8>, black: &Vec<i8>| -> Tensor {
        let mut bitmap:Vec<i64> = vec![0; 64];

        for pos in white{
            bitmap[board120_to_board64(*pos) as usize] = 1;
        }
        
        for pos in black{
            bitmap[board120_to_board64(*pos) as usize] = -1;
        }

        Tensor::of_slice(&bitmap)
    };

    let pawns = to_bitmap(&white.pawns, &black.pawns);
    let knights = to_bitmap(&white.knights, &black.knights);
    let bishops = to_bitmap(&white.bishops, &black.bishops);
    let rooks = to_bitmap(&white.rooks, &black.rooks);
    let queens = to_bitmap(&white.queens, &black.queens);
    let kings = to_bitmap(&white.kings, &black.kings);

    Tensor::cat(&[pawns, knights, bishops, rooks, queens, kings], 0)    

}

fn game_state(game: &GameInfo) -> tch::Tensor{

    let mut game_state:Vec<i64> = vec![0; 69];

    match game.en_passant.last().unwrap(){
        Some(pos) => game_state[board120_to_board64(*pos) as usize] = 1,
        None => ()
    }

    let castling = game.castling.last().unwrap();

    if castling[0] {game_state[64] = 1;}
    if castling[1] {game_state[65] = 1;}
    if castling[2] {game_state[66] = 1;}
    if castling[3] {game_state[67] = 1;}

    match game.turn{
        game::Color::White => game_state[68] = 1,
        game::Color::Black => game_state[68] = -1
    }

    

    Tensor::of_slice(&game_state)
}

fn attacks(game: &mut GameInfo) -> tch::Tensor{
    let mut attacks:Vec<i64> = vec![0; 128];

    let (white_attacks,_) = attack_gen::attack_gen(game, Some(&game::Color::White));
    let (black_attacks,_) = attack_gen::attack_gen(game,Some(&game::Color::Black));

    let board120_to64 = |board: [u8;120]| -> [u8;64]{
        let mut board64: [u8;64] = [0;64];
        for i in 0..64{
            board64[i] = board[board64_to_board120(i as i8) as usize];
        }
        board64
    };

    let white_attacks = board120_to64(white_attacks);
    let black_attacks = board120_to64(black_attacks);

    for i in 0..64{
        if white_attacks[i] > 0{
            attacks[i] = white_attacks[i] as i64;
        }
        if black_attacks[i] > 0{
            attacks[i+64] =  black_attacks[i] as i64 * -1;
        }
    }


    Tensor::of_slice(&attacks)

}


fn model(vs: nn::Path) -> Net{
   
 
    let piece_pos = tch::nn::linear(&vs, 384, 384, Default::default());
    let game_state = tch::nn::linear(&vs, 69, 69, Default::default());
    let attacks = tch::nn::linear(&vs, 128, 128, Default::default());
    let hidden = tch::nn::linear(&vs, 581, 1, Default::default());

    
    Net{piece_pos, game_state, attacks, hidden}

}

pub fn train() -> (){
    let vs = nn::VarStore::new(tch::Device::Cpu);
    let net = model(vs.root());
    let mut opt = nn::Adam::default().build(&vs, 1e-3).unwrap();
   
    for epoch in 0..=N_EPOCHS{
        let mut games = get_training_games();

        for (index,game) in games.iter_mut().enumerate(){

            let mut error:f64 = 0.0;
            let mut last_pred:f64 = 0.0;

            for step in 0..=N_STEPS{

                let mut moves = move_gen::move_gen(game);

                if moves.len() == 0{
                    break;
                }

                let mut best_score = 0.0;
                let mut best_move = moves[0];

                for movement in moves.iter_mut(){

                    make_move::make_move(game, movement);
                    let features = pre_proccess(game);
                    
                    let prediction = net.forward(&features);

                    if game.turn == game::Color::White{
                        if prediction.f_double_value(&[0]).unwrap() < best_score{
                            best_score = prediction.f_double_value(&[0]).unwrap();
                            best_move = *movement;
                        }
                    }else {
                        if prediction.f_double_value(&[0]).unwrap() > best_score{
                            best_score = prediction.f_double_value(&[0]).unwrap();
                            best_move = *movement;
                        }
                    }
                    unmake::unmake_move(game, *movement);

                }
                
                make_move::make_move(game, &mut best_move);
                

                
            }
            
            opt.step();
            opt.zero_grad();

            println!("Epoch: {} Game: {} Loss: {}", epoch, index, error);
        
        }
    }

}

pub fn test(){
    let vs = nn::VarStore::new(tch::Device::Cpu);
    let net = model(vs.root());
    let mut opt = nn::Adam::default().build(&vs, 1e-3).unwrap();
    let mut games = get_training_games();
    let mut features = vec![];
    for game in games.iter_mut(){
        features.push(pre_proccess(game));
    }

    features = features.iter().map(|x| x.to(tch::Device::Cpu)).collect();
    let features = Tensor::stack(&features, 0);
    let predictions = net.forward(&features);

    println!("{}",predictions);
    
}

