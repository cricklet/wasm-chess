#![allow(dead_code)]
#![allow(unused_imports)]
mod chess;
use chess::*;

mod wasm;
use wasm::*;

use async_std::task::sleep;
use std::{
    ops::{Div, Mul},
    sync::Mutex,
    time::Duration,
};
use wasm::log_to_js;
use wasm_bindgen::prelude::*;

use lazy_static::lazy_static;
use web_sys::console;

lazy_static! {
    static ref UCI: Mutex<uci::Uci> = Mutex::new(uci::Uci {
        game: game::Game::from_position_uci(&"position startpos").unwrap(),
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
