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

use async_std::task::sleep;
use std::{
    ops::{Div, Mul},
    sync::Mutex,
    time::Duration,
};
use wasm_bindgen::prelude::*;

use lazy_static::lazy_static;
use web_sys::console;

use crate::{game::Game, uci::Uci};

fn setup_panic_hook() {
    // #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct AsyncCounter {
    counter: Mutex<i32>,
    stop: Mutex<bool>,
}

fn pretend_to_work(ms: i64) {
    let start = chrono::Utc::now();

    let iteration_ms = ms / 4;
    let mut checkpoint = iteration_ms;

    let mut output = "..".to_string();

    log(&". blocking");
    loop {
        let now = chrono::Utc::now();
        if (now - start).num_milliseconds() > ms {
            log(&format!("{} done", output));
            break;
        }

        if (now - start).num_milliseconds() > checkpoint as i64 {
            log(&output);
            checkpoint += iteration_ms;
            output += ".";
        }
    }
}

#[wasm_bindgen]
impl AsyncCounter {
    pub fn new() -> Self {
        setup_panic_hook();
        Self {
            counter: Mutex::new(0),
            stop: Mutex::new(false),
        }
    }
    pub fn count(&self) -> i32 {
        self.counter.lock().unwrap().clone()
    }
    pub async fn start(&self) {
        log(&"starting");
        loop {
            if self.stop.lock().unwrap().clone() {
                break;
            }
            *self.counter.lock().unwrap() += 1;
            log(&format!(
                "> counter: {}",
                self.counter.lock().unwrap().clone()
            ));

            // Pretend to work
            pretend_to_work(100);

            // Relinquish control to js
            sleep(Duration::from_millis(0)).await;
        }
    }
    pub async fn stop(&self) {
        log(&"stopping");
        *self.stop.lock().unwrap() = true;
    }
}

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
    setup_panic_hook();

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

// pub fn process_async(q
