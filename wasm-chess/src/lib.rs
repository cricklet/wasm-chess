mod counter_for_js;
mod perft_for_js;

use wasm_bindgen::prelude::*;

use rust_chess::{
    helpers::{Error, err},
    *, game::Game,
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
        JsError { msg: format!("{}", err) }
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

        UciForJs {
            uci: uci::Uci {
                game: Game::from_position_uci(&"position startpos").unwrap(),
                search: None,
            },
        }
    }
    pub fn handle_line(&mut self, line: &str) -> Result<String, JsError> {
        log_to_js(&format!("handle_line received: {}", line));

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
        self.uci.think().map_err(|e| JsError::from(e))
    }
}
