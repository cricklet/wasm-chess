use std::{sync::Mutex, time::Duration};

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

struct AsyncCounterData {
    counter: i32,
    stop: bool,
}

#[wasm_bindgen]
pub struct AsyncCounter {
    data: Mutex<AsyncCounterData>,
}

pub async fn yield_to_js() {
    async_std::task::sleep(Duration::from_millis(0)).await;
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
impl AsyncCounter {
    pub fn new() -> Self {
        set_panic_hook();
        Self {
            data: Mutex::new(AsyncCounterData {
                counter: 0,
                stop: false,
            }),
        }
    }
    pub fn count(&self) -> i32 {
        let data = self.data.lock().unwrap();
        data.counter
    }

    pub async fn start(&self) {
        log_to_js("start");

        loop {
            {
                let mut data = self.data.lock().unwrap();

                if data.stop {
                    return;
                }
                data.counter += 1;
            }

            // Pretend to work
            pretend_to_work(100);

            // Give control back to the JS event loop
            yield_to_js().await;
        }
    }

    pub async fn stop(&self) {
        log_to_js("stop");
        self.data.lock().unwrap().stop = true;
    }
}

// async fn js_callback(message: AsyncPerftMessage) {
//     match message {
//         AsyncPerftMessage::Log(s) => {
//             log_to_js(&s);
//         }
//         AsyncPerftMessage::Continue => {
//             yield_to_js().await;
//         }
//     }
// }

// #[wasm_bindgen]
// pub struct AsyncPerft {
//     runner: AsyncPerftRunner,
// }

// impl AsyncPerft {
//     pub fn new() -> Self {
//         Self {
//             runner: AsyncPerftRunner::from(|message| async {
//                 match message {
//                     AsyncPerftMessage::Log(s) => {
//                         log_to_js(&s);
//                     }
//                     AsyncPerftMessage::Continue => {
//                         yield_to_js().await;
//                     }
//                 }
//             }),
//         }
//     }
// }
