
use rust_chess::perft::{PerftLoop, PerftLoopResult};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{set_panic_hook, log_to_js};

#[wasm_bindgen]
pub struct PerftForJs {
    data: Option<PerftLoop>,
}

#[wasm_bindgen]
impl PerftForJs {
    pub fn new() -> Self {
        set_panic_hook();
        Self { data: None }
    }

    pub fn count(&self) -> i32 {
        match &self.data {
            Some(data) => data.count as i32,
            None => -1,
        }
    }

    pub fn setup(&mut self, fen: String, max_depth: usize) {
        self.data = Some(PerftLoop::new(&fen, max_depth));
        log_to_js(format!("perft setup {:#?}", self.data.as_ref().unwrap()).as_str());
    }

    pub fn think_and_return_done(&mut self) -> bool {
        let start = chrono::Utc::now();
        match self.data {
            Some(ref mut data) => {
                let result =  data.iterate_loop();
                let elapsed = chrono::Utc::now() - start;
                log_to_js(format!("perft iterated {:#?} in {}ms", result, elapsed.num_milliseconds()).as_str());

                match result {
                    PerftLoopResult::Continue => false,
                    PerftLoopResult::Done => {
                        log_to_js("perft done");
                        true
                    },
                    PerftLoopResult::Interrupted => {
                        panic!("perft interrupted");
                    },
                }
            },
            None => {
                panic!("perft not setup");
            },
        }
    }

    pub fn clear(&mut self) {
        self.data = None;
    }
}
