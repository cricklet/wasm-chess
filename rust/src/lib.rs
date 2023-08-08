#![allow(dead_code)]
#![allow(unused_imports)]
mod wasm_helpers;

pub mod alphabeta;
pub mod bitboard;
pub mod danger;
pub mod evaluation;
pub mod game;
pub mod helpers;

pub mod iterative_traversal;
pub mod moves;
pub mod perft;
pub mod types;
pub mod uci;

use async_std::task::sleep;
use std::{
    ops::{Div, Mul},
    sync::Mutex,
    time::Duration,
};
use wasm_bindgen::prelude::*;
use wasm_helpers::log_to_js;

use lazy_static::lazy_static;
use web_sys::console;

use crate::{game::Game, uci::Uci, wasm_helpers::set_panic_hook};

lazy_static! {
    static ref UCI: Mutex<Uci> = Mutex::new(Uci {
        game: Game::from_position_uci(&"position startpos").unwrap(),
    });
}

#[wasm_bindgen]
pub fn hello() {
    log_to_js("hello from wasm")
}

#[wasm_bindgen]
pub fn process_sync(input: &str) {
    set_panic_hook();

    console::log_1(&format!("> {}", input).into());
    for line in input.split("\n") {
        if line.is_empty() {
            continue;
        }
        for output in UCI.lock().unwrap().handle_line(line) {
            match output {
                Ok(line) => {
                    log_to_js(&line);
                }
                Err(e) => {
                    log_to_js(&format!("Error: {}", e));
                }
            }
        }
    }
}
