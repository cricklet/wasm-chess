#![allow(dead_code)]
#![allow(unused_imports)]

use std::thread::current;

use helpers::{err_result, ErrorResult};
use perft::run_perft;
use uci::Uci;

use crate::{
    game::Game,
    helpers::{indent, prefix},
    perft::run_perft_counting_first_move,
};

pub mod alphabeta;
pub mod bitboard;
pub mod danger;
pub mod evaluation;
pub mod game;
pub mod helpers;
pub mod moves;
pub mod perft;
pub mod types;
pub mod uci;

fn next_stdin() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn run() -> ErrorResult<()> {
    let mut uci = Uci {
        game: Game::from_position_uci(&"position startpos")?,
    };

    loop {
        let input = next_stdin().to_string();
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        for line in input.split("\n") {
            for result in uci.handle_line(line) {
                match result {
                    Ok(line) => println!("{}", line),
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

fn main() {
    run().unwrap();
}
