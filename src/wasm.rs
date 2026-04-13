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
    backend::{CompilerOptions, wasm::CodeGenerator},
    opt::Optimizer,
    parse,
};
use js_sys::{BigInt, Function, Map, Number, Reflect, Undefined, WebAssembly};
use std::time::Instant;
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::JsFuture;

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
    code: &str,
    read: Function<fn() -> Number>,
    write: Function<fn(Number) -> Undefined>,
    finisher: Function<fn(BigInt) -> Undefined>,
) {
    let opts = CompilerOptions {
        opt_level: 8,
        ..Default::default()
    };

    let program = parse(code);
    let program = Optimizer::new(&opts, program).run_all().finish();
    let module = CodeGenerator::run(&opts, &program);

    let obj = Map::new().set(
        &"bf".into(),
        &Map::new()
            .set(&"putchar".into(), &write)
            .set(&"getchar".into(), &read),
    );

    let module = JsFuture::from(WebAssembly::instantiate_streaming(&module.into(), &obj))
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
