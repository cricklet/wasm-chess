#![allow(dead_code)]
#![allow(unused_imports)]

use helpers::{err_result, ErrorResult};
use perft::run_perft;

use crate::{
    game::Game,
    helpers::{indent, prefix},
};

pub mod bitboards;
pub mod game;
pub mod helpers;
pub mod moves;
pub mod perft;
pub mod types;

fn next_stdin() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn run() -> ErrorResult<()> {
    let mut current_game: Option<Game> = None;

    loop {
        let input = next_stdin();
        if input.is_empty() {
            continue;
        } else if input.starts_with("position") {
            current_game = Some(Game::from_position_uci(&input)?);
            println!("{}", prefix(&format!("{}", current_game.unwrap()), "> "));
        } else if input.starts_with("go perft") {
            let depth = input["go perft".len()..].trim();
            let depth = match depth.parse::<usize>() {
                Ok(depth) => depth,
                Err(_) => {
                    return err_result(&format!("invalid depth for '{}'", input));
                }
            };
            let (perft_overall, perft_per_move) = run_perft(&current_game.unwrap(), depth)?;
            perft_per_move.iter().for_each(|(mv, count)| {
                println!("{}: {}", mv.to_uci(), count);
            });
            println!("Nodes searched: {}", perft_overall);
        } else if input == "stop" {
            break;
        } else {
            return err_result(format!("Unknown command: '{}'", input).as_str());
        }
    }

    Ok(())
}

fn main() {
    run().unwrap();
}
