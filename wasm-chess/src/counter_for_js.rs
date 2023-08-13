use wasm_bindgen::prelude::wasm_bindgen;

use crate::{log_to_js, set_panic_hook};

#[wasm_bindgen]
pub struct CounterForJs {
    counter: i32,
}

fn pretend_to_work(ms: i64) {
    let start = chrono::Utc::now();

    let iteration_ms = ms / 4;
    let mut checkpoint = iteration_ms;

    let mut output = ".".to_string();

    loop {
        let now = chrono::Utc::now();
        if (now - start).num_milliseconds() > ms {
            log_to_js(&format!("{} done", output));
            break;
        }

        if (now - start).num_milliseconds() > checkpoint {
            log_to_js(&output);
            checkpoint += iteration_ms;
            output += ".";
        }
    }
}

#[wasm_bindgen]
impl CounterForJs {
    pub fn new() -> Self {
        set_panic_hook();
        Self { counter: 0 }
    }

    pub fn think(&mut self) {
        self.counter += 1;
        pretend_to_work(100);
    }

    pub fn count(&self) -> i32 {
        self.counter
    }
}
