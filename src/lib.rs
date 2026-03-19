pub mod backend;
pub mod interp;
pub mod opt;

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

use serde::Serialize;

pub const TAPE_SIZE: usize = u16::MAX as usize;

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
