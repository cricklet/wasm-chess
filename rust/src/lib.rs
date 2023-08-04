#![allow(dead_code)]
#![allow(unused_imports)]
mod wasm_utils;

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

use std::sync::Mutex;
use wasm_bindgen::prelude::*;

use lazy_static::lazy_static;
use web_sys::console;

use crate::{game::Game, uci::Uci};

#[wasm_bindgen(js_namespace = ["globalThis", "BindingsJs"])]
extern "C" {
    #[wasm_bindgen()]
    fn log(s: &str);
}

lazy_static! {
    static ref UCI: Mutex<Uci> = Mutex::new(Uci {
        game: Game::from_position_uci(&"position startpos").unwrap(),
    });
}

#[wasm_bindgen]
pub fn hello() {
    log("hello from wasm")
}

#[wasm_bindgen]
pub fn process_sync(input: &str) {
    console::log_1(&format!("> {}", input).into());
    for line in input.split("\n") {
        if line.is_empty() {
            continue;
        }
        for output in UCI.lock().unwrap().handle_line(line) {
            match output {
                Ok(line) => {
                    log(&line);
                }
                Err(e) => {
                    log(&format!("Error: {}", e));
                }
            }
        }
    }
}

pub fn process_async(input: &str) {}
