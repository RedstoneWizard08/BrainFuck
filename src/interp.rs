use std::io::{Read, Write};

use crate::{TAPE_SIZE, optimizer::OptAction};

pub struct ProgramState {
    tape: [u8; TAPE_SIZE],
    tape_ptr: i64,
}

const TAPE_SIZE_I: i64 = TAPE_SIZE as i64;

impl ProgramState {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            tape: [0; TAPE_SIZE],
            tape_ptr: 0,
        }
    }

    #[inline(always)]
    pub fn add(&mut self, amnt: i64) {
        self.tape[wrap_to_index(self.tape_ptr)] =
            wrapping_conv((self.tape[wrap_to_index(self.tape_ptr)] as i64) + amnt);
    }

    #[inline(always)]
    pub fn move_ptr(&mut self, amnt: i64) {
        self.tape_ptr = wrap_to_index(self.tape_ptr + amnt) as i64;
    }

    #[inline(always)]
    pub fn get(&self) -> u8 {
        self.tape[wrap_to_index(self.tape_ptr)]
    }

    #[inline(always)]
    pub fn set(&mut self, value: u8) {
        self.tape[wrap_to_index(self.tape_ptr)] = value;
    }
}

pub fn eval<W: Write, R: Read>(
    insn: &OptAction,
    output: &mut W,
    input: &mut R,
    state: &mut ProgramState,
) {
    match insn {
        OptAction::Output => {
            output.write(&[state.get()]).unwrap();
        }

        OptAction::Input => {
            let mut buf = [0u8; 1];

            input.read(&mut buf).unwrap();
            state.set(buf[0]);
        }

        OptAction::Loop(actions) => {
            while state.get() != 0 {
                for insn in actions {
                    eval(insn, output, input, state);
                }
            }
        }

        OptAction::Noop => (),

        OptAction::AddValue(v) => {
            state.add(*v);
        }

        OptAction::SetValue(v) => {
            state.set(wrapping_conv(*v));
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
    }
}

fn wrapping_conv(a: i64) -> u8 {
    let a = if a < 0 { i64::MAX + a } else { a };

    u8::try_from(a % i64::from(u8::MAX)).expect(&format!("Failed to convert i64 to u8: {a}"))
}

fn wrap_to_index(a: i64) -> usize {
    if a > TAPE_SIZE_I {
        TAPE_SIZE - (a % TAPE_SIZE_I) as usize
    } else if a < 0 {
        (TAPE_SIZE_I + (a % TAPE_SIZE_I)) as usize
    } else {
        a as usize
    }
}

pub fn interpret<W: Write, R: Read>(program: &Vec<OptAction>, output: &mut W, input: &mut R) {
    let mut state = ProgramState::new();

    for insn in program {
        eval(insn, output, input, &mut state);
    }
}
