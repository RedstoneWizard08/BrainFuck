//! Direct interpreter for Brainf*ck programs.
//!
//! This module provides an interpreter that directly executes Brainf*ck
//! programs without code generation, useful for rapid prototyping and testing.
//!
//! # Examples
//!
//! The interpreter can be used to directly execute Brainf*ck programs:
//!
//! ```no_run
//! use bf::parse;
//! // Parse a simple program that outputs 'A' (65)
//! let program = "++++++++[>++++++++<-]>.";
//! let actions = parse(program);
//! // The interpreter would execute this directly
//! ```

use crate::opt::action::{OptAction, ValueAction};
use std::io::{Read, Write};

/// The size of the memory tape available to Brainf*ck programs
const TAPE_SIZE: usize = u16::MAX as usize;
const TAPE_SIZE_I: i64 = TAPE_SIZE as i64;

/// Represents the execution state of a Brainf*ck program.
///
/// Holds the memory tape and the data pointer during program execution.
///
/// # Examples
///
/// A tape starts at position 0 with all cells initialized to 0
struct ProgramState {
    /// The memory tape (initialized to all zeros)
    tape: [u8; TAPE_SIZE + 1],
    /// The current position on the tape
    tape_ptr: i64,
}

impl ProgramState {
    /// Creates a new program state with an empty tape and starting pointer at 0.
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            tape: [0; TAPE_SIZE + 1],
            tape_ptr: 0,
        }
    }

    /// Adds a value to the current cell on the tape.
    #[inline(always)]
    pub const fn add(&mut self, amnt: i64) {
        let ptr = wrap_to_index(self.tape_ptr);

        self.tape[ptr] = wrapping_conv((self.tape[ptr] as i64) + amnt);
    }

    /// Adds a value to a cell at an offset from the current pointer.
    #[inline(always)]
    pub const fn add_offset(&mut self, amnt: i64, offset: i64) {
        let ptr = wrap_to_index(self.tape_ptr + offset);

        self.tape[ptr] = wrapping_conv((self.tape[ptr] as i64) + amnt);
    }

    #[inline(always)]
    pub const fn move_ptr(&mut self, amnt: i64) {
        self.tape_ptr = wrap_to_index(self.tape_ptr + amnt) as i64;
    }

    #[inline(always)]
    pub const fn get(&self) -> u8 {
        self.tape[wrap_to_index(self.tape_ptr)]
    }

    #[inline(always)]
    pub const fn get_offset(&self, offset: i64) -> u8 {
        self.tape[wrap_to_index(self.tape_ptr + offset)]
    }

    #[inline(always)]
    pub const fn set(&mut self, value: u8) {
        self.tape[wrap_to_index(self.tape_ptr)] = value;
    }

    #[inline(always)]
    pub const fn set_offset(&mut self, value: u8, offset: i64) {
        self.tape[wrap_to_index(self.tape_ptr + offset)] = value;
    }
}

fn eval<W: Write, R: Read>(
    insn: &OptAction,
    output: &mut W,
    input: &mut R,
    state: &mut ProgramState,
) {
    match insn {
        OptAction::Noop => (),

        OptAction::Value(it) => match it {
            ValueAction::Output => {
                let _ = output.write(&[state.get()]);
            }

            ValueAction::BulkPrint(n) => {
                for _ in 0..*n {
                    let _ = output.write(&[state.get()]);
                }
            }

            ValueAction::Input => {
                let mut buf = [0u8; 1];

                let _ = input.read(&mut buf);
                state.set(buf[0]);
            }

            ValueAction::AddValue(v) => {
                state.add(*v);
            }

            ValueAction::SetValue(v) => {
                state.set(wrapping_conv(*v));
            }
        },

        OptAction::OffsetValue(it, offset) => match it {
            ValueAction::Output => {
                output.write(&[state.get_offset(*offset)]).unwrap();
            }

            ValueAction::BulkPrint(n) => {
                for _ in 0..*n {
                    output.write(&[state.get_offset(*offset)]).unwrap();
                }
            }

            ValueAction::Input => {
                let mut buf = [0u8; 1];

                input.read(&mut buf).unwrap();
                state.set_offset(buf[0], *offset);
            }

            ValueAction::AddValue(v) => {
                state.add_offset(*v, *offset);
            }

            ValueAction::SetValue(v) => {
                state.set_offset(wrapping_conv(*v), *offset);
            }
        },

        OptAction::Loop(actions) => {
            while state.get() != 0 {
                for insn in actions {
                    eval(insn, output, input, state);
                }
            }
        }

        OptAction::MovePtr(v) => {
            state.move_ptr(*v);
        }

        OptAction::SetAndMove(v, o) => {
            let w = wrapping_conv(*v);

            state.set(w);
            state.move_ptr(*o);
        }

        OptAction::AddAndMove(v, o) => {
            state.add(*v);
            state.move_ptr(*o);
        }

        OptAction::CopyLoop(v) => {
            let ptr = state.tape_ptr;
            let cur = i64::from(state.tape[wrap_to_index(state.tape_ptr)]);

            for (o, v) in v {
                let pos = wrap_to_index(ptr + *o);
                let val = i64::from(state.tape[pos]);

                state.tape[pos] = wrapping_conv(val + cur * *v);
            }

            state.tape[wrap_to_index(state.tape_ptr)] = 0;
        }

        OptAction::Scan(skip) => {
            while state.tape[wrap_to_index(state.tape_ptr)] != 0 {
                state.tape_ptr += skip;
            }
        }
    }
}

#[inline(always)]
pub const fn wrapping_conv(a: i64) -> u8 {
    let a = if a < 0 { i64::MAX + a } else { a };

    (a % (u8::MAX as i64)) as u8
}

#[inline(always)]
pub const fn wrap_to_index(a: i64) -> usize {
    if TAPE_SIZE.trailing_zeros() == 0 {
        (a & TAPE_SIZE_I) as usize
    } else {
        if a > TAPE_SIZE_I {
            TAPE_SIZE - (a % TAPE_SIZE_I) as usize
        } else if a < 0 {
            (TAPE_SIZE_I + (a % TAPE_SIZE_I)) as usize
        } else {
            a as usize
        }
    }
}

pub fn interpret<W: Write, R: Read>(program: &Vec<OptAction>, output: &mut W, input: &mut R) {
    let mut state = ProgramState::new();

    for insn in program {
        eval(insn, output, input, &mut state);
    }
}
