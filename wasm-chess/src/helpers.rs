
use rust_chess::perft::{PerftLoop, PerftLoopResult};
use wasm_bindgen::prelude::wasm_bindgen;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    //
    // Note, if this is shared between multiple targets, you can use #[cfg(feature)]
    // to turn turn this off: eg #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_namespace = ["globalThis", "BindingsJs"])]
extern "C" {
    #[wasm_bindgen()]
    pub fn log_to_js(s: &str);
}

#[wasm_bindgen]
pub struct CounterForJs {
    counter: i32,
}

// pub async fn yield_to_js() {
//     async_std::task::sleep(Duration::from_millis(0)).await;
// }

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
        Self {
            counter: 0,
        }
    }

    pub fn think(&mut self) {
        self.counter += 1;
        pretend_to_work(100);
    }

    pub fn count(&self) -> i32 {
        self.counter
    }
}

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
