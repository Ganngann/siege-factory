use siege_factory::run;

fn main() {
    run();
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
fn start() {
    run();
}
