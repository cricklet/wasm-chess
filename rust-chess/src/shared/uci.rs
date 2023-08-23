use itertools::Itertools;
use std::{cell::RefCell, iter, rc::Rc, sync::Mutex};

use crate::{
    alphabeta::{AlphaBetaStack, LoopResult},
    bitboard::warm_magic_cache,
    iterative_deepening::{IterativeSearch, IterativeSearchOptions},
    transposition_table::TranspositionTable, helpers::Joinable,
};

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
    pub search: Option<IterativeSearch>,
    pub tt: Rc<RefCell<TranspositionTable>>,
}

impl Uci {
    pub fn new() -> Self {
        Self {
            game: Game::from_position_uci(&"position startpos").unwrap(),
            search: None,
            tt: Rc::new(RefCell::new(TranspositionTable::new())),
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
            let search = IterativeSearch::new(
                self.game,
                IterativeSearchOptions {
                    transposition_table: Some(self.tt.clone()),
                    ..IterativeSearchOptions::default()
                },
            )?;
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
                Some((best_move, response_moves)) => Ok(format!(
                    "bestmove {} ponder {}",
                    best_move,
                    response_moves.iter().map(|v| v.to_string()).join(" ")
                )),
                None => Ok("bestmove (none)".to_string()),
            }
        } else {
            Ok("".to_string())
        }
    }

    pub fn think(&mut self) -> ErrorResult<String> {
        let mut output: Vec<String> = vec![];
        for _ in 0..100_000 {
            if let Some(search) = &mut self.search {
                search.iterate(&mut |line| output.push(line.to_string()))?;
            }
        }

        Ok(output.join("\n"))
    }
}

#[test]
fn test_match_50ms() {
    let mut uci = Uci::new();
    let mut moves: Vec<String> = vec![];

    loop {
        let position_uci_line = format!("position startpos moves {}", moves.join(" "));
        println!("{}", position_uci_line);
        uci.handle_line(position_uci_line.as_str()).unwrap();

        let start = std::time::Instant::now();
        uci.handle_line("go").unwrap();

        let mut log = vec![];

        loop {
            log.push(uci.think().unwrap());
            if start.elapsed().as_millis() > 50 {
                break;
            }
        }

        println!(
            "{}",
            log.iter().filter(|&l| !l.is_empty()).collect::<Vec<_>>().join_vec(", ")
        );

        let result = uci.finish_search().unwrap();
        println!("{}", result);
        println!("{}", uci.handle_line("d").unwrap());

        let bestmove = result.split_whitespace().nth(1).unwrap();
        if bestmove.contains("none") {
            break;
        }

        moves.push(bestmove.to_string());
    }
}

#[test]
fn test_match_100ms() {
    let mut uci = Uci::new();
    let mut moves: Vec<String> = vec![];

    loop {
        let position_uci_line = format!("position startpos moves {}", moves.join(" "));
        println!("{}", position_uci_line);
        uci.handle_line(position_uci_line.as_str()).unwrap();

        let start = std::time::Instant::now();
        uci.handle_line("go").unwrap();

        let mut log = vec![];

        loop {
            log.push(uci.think().unwrap());
            if start.elapsed().as_millis() > 100 {
                break;
            }
        }

        println!(
            "{}",
            log.iter().filter(|&l| !l.is_empty()).collect::<Vec<_>>().join_vec(", ")
        );

        let result = uci.finish_search().unwrap();
        println!("{}", result);
        println!("{}", uci.handle_line("d").unwrap());

        let bestmove = result.split_whitespace().nth(1).unwrap();
        if bestmove.contains("none") {
            break;
        }

        moves.push(bestmove.to_string());
    }
}

#[test]
fn test_match_1000ms() {
    let mut uci = Uci::new();
    let mut moves: Vec<String> = vec![];

    loop {
        let position_uci_line = format!("position startpos moves {}", moves.join(" "));
        println!("{}", position_uci_line);
        uci.handle_line(position_uci_line.as_str()).unwrap();

        let start = std::time::Instant::now();
        uci.handle_line("go").unwrap();

        let mut log = vec![];

        loop {
            log.push(uci.think().unwrap());
            if start.elapsed().as_millis() > 1000 {
                break;
            }
        }

        println!(
            "{}",
            log.iter().filter(|&l| !l.is_empty()).collect::<Vec<_>>().join_vec(", ")
        );

        let result = uci.finish_search().unwrap();
        println!("{}", result);
        println!("{}", uci.handle_line("d").unwrap());

        let bestmove = result.split_whitespace().nth(1).unwrap();
        if bestmove.contains("none") {
            break;
        }

        moves.push(bestmove.to_string());
    }
}
