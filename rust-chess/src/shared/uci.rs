use itertools::Itertools;
use std::{iter, sync::Mutex};

use crate::search::{LoopResult, Search};

use super::{
    game::Game,
    helpers::{err_result, ErrorResult},
    perft::run_perft_counting_first_move,
};

pub struct UciAsync {
    game: Mutex<Game>,
}

pub struct Uci {
    pub game: Game,
    pub search: Option<Search>,
}

impl Uci {
    pub fn new() -> Self {
        Self {
            game: Game::from_position_uci(&"position startpos").unwrap(),
            search: None,
        }
    }
    pub fn handle_line(&mut self, line: &str) -> ErrorResult<String> {
        if line.starts_with("position") {
            let game = Game::from_position_uci(line);
            if let Err(e) = &game {
                return Err(e.clone());
            }
            self.game = game.unwrap();
            Ok(format!("{:?}", self.game))
        } else if line.starts_with("go perft") {
            let depth = line["go perft".len()..].trim();
            let depth = match depth.parse::<usize>() {
                Ok(depth) => depth,
                Err(_) => {
                    return err_result(&format!("invalid depth for '{}'", line));
                }
            };
            let perft_result = run_perft_counting_first_move(&self.game, depth);
            if let Err(e) = &perft_result {
                return Err(e.clone());
            }
            let (perft_overall, perft_per_move) = perft_result.unwrap();
            let perft_per_move = perft_per_move
                .into_iter()
                .map(|(mv, count)| format!("{}: {}", mv, count));
            let perft_output = format!(
                "{}\nNodes searched: {}",
                perft_per_move.into_iter().join("\n"),
                perft_overall,
            );
            Ok(perft_output)
        } else if line == "d" {
            let debug_str = format!("{}\nFen: {}", self.game, self.game.to_fen());
            Ok(debug_str)
        } else if line == "go" {
            let search = Search::new(self.game)?;
            self.search = Some(search);
            Ok("".to_string())
        } else if line == "stop" {
            self.finish_search()
        } else {
            err_result(&format!("Unknown command: '{}'", line))
        }
    }

    fn finish_search(&mut self) -> ErrorResult<String> {
        if let Some(search) = &mut self.search {
            let best_move = search.bestmove();
            self.search = None;

            match best_move {
                Some((best_move, _)) => Ok(format!("bestmove {}", best_move)),
                None => Ok("bestmove (none)".to_string()),
            }
        } else {
            Ok("".to_string())
        }
    }

    pub fn think(&mut self) -> ErrorResult<String> {
        if let Some(search) = &mut self.search {
            let result = search.iterate()?;

            match result {
                LoopResult::Done => self.finish_search(),
                _ => Ok("".to_string()),
            }
        } else {
            Ok("".to_string())
        }
    }
}
