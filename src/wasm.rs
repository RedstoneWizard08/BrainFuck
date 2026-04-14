//! WebAssembly code generation and browser integration.
//!
//! This module provides compilation to WebAssembly and browser interop
//! for executing Brainf*ck programs in web environments.
//!
//! # Examples
//!
//! Compiling and running a Brainf*ck program in the browser:
//!
//! ```javascript
//! import { compile_run } from './pkg/index.js';
//!
//! const program = "++++++++[>++++++++<-]>.";
//! const result = await compile_run(program, null, null);
//! console.log("Execution time:", result.time_ms);
//! ```

#[cfg(not(feature = "wasm"))]
compile_error!("Web interop requires the WASM backend!");

use crate::{
    backend::{CompilerOptions, Optimization, wasm::CodeGenerator},
    opt::v2::optimize_v2,
    parse,
};
use js_sys::{BigInt, Function, Number, Object, Reflect, Undefined, WebAssembly};
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use web_time::Instant;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
}

/// Compiles and runs a Brainf*ck program in the browser.
///
/// This async function compiles a Brainf*ck source code string to WebAssembly
/// and executes it with optional I/O callbacks for interacting with JavaScript.
///
/// # Parameters
///
/// * `code` - The Brainf*ck source code as a JavaScript string
/// * `get_char` - Optional callback function for input operations
/// * `put_char` - Optional callback function for output operations
///
/// # Returns
///
/// A Promise that resolves to a JavaScript object containing execution results
#[wasm_bindgen]
pub async fn compile_run(
    code: String,
    read: &Function<fn() -> Number>,
    write: &Function<fn(Number) -> Undefined>,
    finisher: &Function<fn(BigInt) -> Undefined>,
) {
    let opts = CompilerOptions {
        opt_level: 8,
        no_optimize: vec![Optimization::Scanners],
        ..Default::default()
    };

    let program = parse(&code);
    let program = optimize_v2(&program, &opts);
    let module = CodeGenerator::run(&opts, &program);
    let bf = Object::new();

    Reflect::set(&bf, &"putchar".into(), &write).unwrap();
    Reflect::set(&bf, &"getchar".into(), &read).unwrap();

    let obj = Object::new();

    Reflect::set(&obj, &"bf".into(), &bf).unwrap();

    let module = WebAssembly::instantiate_buffer(&module, &obj)
        .await
        .unwrap();

    let inst: WebAssembly::Instance = Reflect::get(&module, &"instance".into())
        .unwrap()
        .dyn_into()
        .unwrap();

    let exports = inst.exports();

    let start = Reflect::get(exports.as_ref(), &"_start".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap();

    let now = Instant::now();

    start.call0(&JsValue::undefined()).unwrap();

    finisher
        .call1(&JsValue::undefined(), &now.elapsed().as_millis().into())
        .unwrap();
}

#[wasm_bindgen]
pub fn compile(code: String) -> Vec<u8> {
    let opts = CompilerOptions {
        opt_level: 8,
        no_optimize: vec![Optimization::Scanners],
        ..Default::default()
    };

    let program = parse(&code);
    let program = optimize_v2(&program, &opts);

    CodeGenerator::run(&opts, &program)
}
