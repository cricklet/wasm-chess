mod counter_for_js;
mod perft_for_js;

use wasm_bindgen::prelude::*;

use rust_chess::{
    bitboard::warm_magic_cache,
    helpers::{err, Error},
    *,
};

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
pub struct UciForJs {
    uci: uci::Uci,
}

pub struct JsError {
    msg: String,
}

impl JsError {
    pub fn from(err: Error) -> Self {
        log_to_js(&format!("error in rust-wasm: {:?}", err));
        JsError {
            msg: format!("{}", err),
        }
    }
}

impl Into<JsValue> for JsError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.msg)
    }
}

#[wasm_bindgen]
impl UciForJs {
    pub fn new() -> Self {
        set_panic_hook();

        log_to_js("warming magic cache");
        warm_magic_cache();
        log_to_js("... done");

        UciForJs {
            uci: uci::Uci::new(log_to_js),
        }
    }
    pub fn handle_line(&mut self, line: &str) -> Result<String, JsError> {
        log_to_js(&format!("handle_line({})", line));

        if line.contains("\n") {
            return Err(JsError::from(err(&format!("{} contains newline", line))));
        }

        let output = self.uci.handle_line(line);
        match output {
            Ok(output) => Ok(output),
            Err(e) => Err(JsError::from(e)),
        }
    }

    pub fn think(&mut self) -> Result<String, JsError> {
        let start = chrono::Utc::now();
        let result = self.uci.think().map_err(|e| JsError::from(e))?;

        let elapsed = chrono::Utc::now() - start;
        if elapsed > chrono::Duration::milliseconds(1) {
            let ms = elapsed.num_milliseconds();
            if ms < 10 {
                log_to_js("think() for < 10ms");
            } else if ms < 40 {
                log_to_js("think() for < 40ms");
            } else if ms < 200 {
                log_to_js("think() for < 200ms");
            } else {
                log_to_js(&format!("think() for {}ms", ms));
            }
        }

        Ok(result)
    }
}
