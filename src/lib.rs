// I mean, we already require the nightly compiler, so doing this for the cryaboutit module isn't that big of a deal.
#![allow(incomplete_features, internal_features)]
#![feature(sized_type_properties, generic_const_exprs, core_intrinsics)]
// Anyways back to the main event
#![cfg_attr(feature = "jvm", feature(const_trait_impl, const_ops))]

pub mod backend;
pub mod opt;

#[cfg(feature = "interp")]
pub mod interp;

#[cfg(feature = "web")]
pub mod wasm;

#[cfg(feature = "cranelift")]
pub mod link;

#[cfg(feature = "cranelift")]
pub mod linker;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "testing")]
pub mod testing;

#[allow(unused)]
mod cryaboutit;

use serde::Serialize;

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

pub fn parse(input: &str) -> Vec<Action> {
    let mut stack: Vec<Vec<Action>> = Vec::new();

    stack.push(Vec::new());

    for ch in input.chars() {
        let cur = stack.last_mut().unwrap();

        match ch {
            '>' => cur.push(Action::Right),
            '<' => cur.push(Action::Left),
            '+' => cur.push(Action::Inc),
            '-' => cur.push(Action::Dec),
            '.' => cur.push(Action::Output),
            ',' => cur.push(Action::Input),
            '[' => stack.push(Vec::new()),

            ']' => {
                let last = stack.pop().unwrap();
                let cur = stack.last_mut().unwrap();

                cur.push(Action::Loop(last));
            }

            _ => (),
        }
    }

    if stack.len() != 1 {
        panic!("Stack length was {} when it should be 1!", stack.len());
    }

    stack.remove(0)
}
