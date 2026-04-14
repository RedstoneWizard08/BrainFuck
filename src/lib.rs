//! Brainf*ck language compiler and runtime library.
//!
//! This crate provides a complete implementation of a Brainf*ck compiler supporting
//! multiple backends (LLVM, Cranelift, JVM, WebAssembly, ASM), optimizations, and
//! various code generation targets.

// I mean, we already require the nightly compiler, so doing this for the cryaboutit module isn't that big of a deal.
#![allow(incomplete_features, internal_features)]
#![feature(sized_type_properties, generic_const_exprs, core_intrinsics)]
// Anyways back to the main event
#![cfg_attr(feature = "jvm", feature(const_trait_impl, const_ops))]

/// Code generation backends for various target architectures and languages
pub mod backend;
/// Optimization passes for improving generated code
pub mod opt;

/// Interpreter for executing Brainf*ck code directly
#[cfg(feature = "interp")]
pub mod interp;

/// Interpreter for executing Brainf*ck code directly, with unsafe optimizations
#[cfg(feature = "interp")]
pub mod interp_unsafe;

/// WebAssembly code generation support
#[cfg(feature = "web")]
pub mod wasm;

/// LLVM linking and code generation
#[cfg(feature = "cranelift")]
pub mod link;

/// Platform-specific linker implementations
#[cfg(feature = "cranelift")]
pub mod linker;

/// Command-line interface for the compiler
#[cfg(feature = "cli")]
pub mod cli;

/// Testing utilities and fixtures
#[cfg(feature = "testing")]
pub mod testing;

/// Internal cryptic utilities module
#[allow(unused)]
mod cryaboutit;

use serde::Serialize;

/// Represents the fundamental actions in a Brainf*ck program.
///
/// Each variant corresponds to a basic Brainf*ck operation:
/// - `Right`: Move the pointer to the right (`>`)
/// - `Left`: Move the pointer to the left (`<`)
/// - `Inc`: Increment the value at the pointer (`+`)
/// - `Dec`: Decrement the value at the pointer (`-`)
/// - `Output`: Output the value at the pointer (`.`)
/// - `Input`: Read a value from input (`,`)
/// - `Loop`: A loop construct `[...]` containing nested actions
///
/// # Examples
///
/// Creating actions manually:
///
/// ```
/// use bf::Action;
///
/// let actions = vec![
///     Action::Inc,
///     Action::Output,
/// ];
/// ```
///
/// Creating a loop with nested actions:
///
/// ```
/// use bf::Action;
///
/// let loop_body = vec![
///     Action::Inc,
///     Action::Right,
/// ];
/// let loop_action = Action::Loop(loop_body);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Action {
    Right,
    Left,
    Inc,
    Dec,
    Output,
    Input,
    Loop(Vec<Action>),
}

/// Parses a Brainf*ck program string into an abstract syntax tree of Actions.
///
/// This function scans the input string character by character and builds a nested
/// vector structure representing the program's actions, properly handling loop nesting.
///
/// # Arguments
///
/// * `input` - A string containing Brainf*ck source code
///
/// # Returns
///
/// A vector of Actions representing the parsed program
///
/// # Panics
///
/// Panics if the bracket nesting is unbalanced (more closing brackets than opening)
///
/// # Examples
///
/// Parse a simple increment and output program:
///
/// ```
/// use bf::parse;
/// use bf::Action;
///
/// let program = "+++.";
/// let actions = parse(program);
/// assert_eq!(actions, vec![
///     Action::Inc,
///     Action::Inc,
///     Action::Inc,
///     Action::Output,
/// ]);
/// ```
///
/// Parse a program with loops:
///
/// ```
/// use bf::parse;
/// use bf::Action;
///
/// let program = "[>+<-]";
/// let actions = parse(program);
/// assert_eq!(actions.len(), 1);
/// match &actions[0] {
///     Action::Loop(body) => {
///         assert_eq!(body.len(), 4);
///     }
///     _ => panic!("Expected a loop"),
/// }
/// ```
#[cfg(not(feature = "unsafe-speed"))]
pub fn parse(input: &str) -> Vec<Action> {
    let mut stack: Vec<Vec<Action>> = Vec::new();

    stack.push(Vec::new());

    for ch in input.chars() {
        match ch {
            '>' => stack.last_mut().unwrap().push(Action::Right),
            '<' => stack.last_mut().unwrap().push(Action::Left),
            '+' => stack.last_mut().unwrap().push(Action::Inc),
            '-' => stack.last_mut().unwrap().push(Action::Dec),
            '.' => stack.last_mut().unwrap().push(Action::Output),
            ',' => stack.last_mut().unwrap().push(Action::Input),
            '[' => stack.push(Vec::new()),

            ']' => {
                let last = stack.pop().unwrap();

                stack.last_mut().unwrap().push(Action::Loop(last));
            }

            _ => (),
        }
    }

    if stack.len() != 1 {
        panic!("Stack length was {} when it should be 1!", stack.len());
    }

    stack.remove(0)
}

/// Parses a Brainf*ck program string into an abstract syntax tree of Actions.
///
/// This function scans the input string character by character and builds a nested
/// vector structure representing the program's actions, properly handling loop nesting.
///
/// # Arguments
///
/// * `input` - A string containing Brainf*ck source code
///
/// # Returns
///
/// A vector of Actions representing the parsed program
///
/// # Panics
///
/// Panics if the bracket nesting is unbalanced (more closing brackets than opening)
///
/// # Examples
///
/// Parse a simple increment and output program:
///
/// ```
/// use bf::parse;
/// use bf::Action;
///
/// let program = "+++.";
/// let actions = parse(program);
/// assert_eq!(actions, vec![
///     Action::Inc,
///     Action::Inc,
///     Action::Inc,
///     Action::Output,
/// ]);
/// ```
///
/// Parse a program with loops:
///
/// ```
/// use bf::parse;
/// use bf::Action;
///
/// let program = "[>+<-]";
/// let actions = parse(program);
/// assert_eq!(actions.len(), 1);
/// match &actions[0] {
///     Action::Loop(body) => {
///         assert_eq!(body.len(), 4);
///     }
///     _ => panic!("Expected a loop"),
/// }
/// ```
#[cfg(feature = "unsafe-speed")]
pub fn parse(input: &str) -> Vec<Action> {
    let mut stack: Vec<Vec<Action>> = Vec::new();

    stack.push(Vec::new());

    for ch in input.chars() {
        let pos = stack.len() - 1;

        match ch {
            '>' => unsafe { stack.get_unchecked_mut(pos) }.push(Action::Right),
            '<' => unsafe { stack.get_unchecked_mut(pos) }.push(Action::Left),
            '+' => unsafe { stack.get_unchecked_mut(pos) }.push(Action::Inc),
            '-' => unsafe { stack.get_unchecked_mut(pos) }.push(Action::Dec),
            '.' => unsafe { stack.get_unchecked_mut(pos) }.push(Action::Output),
            ',' => unsafe { stack.get_unchecked_mut(pos) }.push(Action::Input),
            '[' => stack.push(Vec::new()),

            ']' => {
                let last = stack.pop().unwrap();

                unsafe { stack.get_unchecked_mut(pos - 1) }.push(Action::Loop(last));
            }

            _ => (),
        }
    }

    stack.remove(0)
}
