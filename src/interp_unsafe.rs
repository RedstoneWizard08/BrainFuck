use crate::opt::action::{OptAction, ValueAction};
use std::io::{Read, Write};

const TAPE_SIZE: usize = 65536;

struct ProgramState {
    _tape: [u8; TAPE_SIZE],
    tape_ptr: *mut u8,
}

#[allow(unsafe_op_in_unsafe_fn)]
impl ProgramState {
    #[inline(always)]
    pub const unsafe fn new() -> Self {
        let mut tape = [0; TAPE_SIZE];

        Self {
            tape_ptr: tape.as_mut_ptr(),
            _tape: tape,
        }
    }

    #[inline(always)]
    pub const unsafe fn add(&mut self, amnt: i64) {
        *self.tape_ptr = ((*self.tape_ptr as i64) + amnt) as u8;
    }

    #[inline(always)]
    pub const unsafe fn add_offset(&mut self, amnt: i64, offset: i64) {
        let ptr = self.tape_ptr.wrapping_offset(offset as isize);

        *ptr = ((*ptr as i64) + amnt) as u8;
    }

    #[inline(always)]
    pub const unsafe fn move_ptr(&mut self, amnt: i64) {
        self.tape_ptr = self.tape_ptr.wrapping_offset(amnt as isize);
    }

    #[inline(always)]
    pub const unsafe fn get(&self) -> u8 {
        *self.tape_ptr
    }

    #[inline(always)]
    pub const unsafe fn get_offset(&self, offset: i64) -> u8 {
        *self.tape_ptr.wrapping_offset(offset as isize)
    }

    #[inline(always)]
    pub const unsafe fn set(&mut self, value: u8) {
        *self.tape_ptr = value;
    }

    #[inline(always)]
    pub const unsafe fn set_offset(&mut self, value: u8, offset: i64) {
        *self.tape_ptr.wrapping_offset(offset as isize) = value;
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn eval<W: Write, R: Read>(
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

            ValueAction::AddValue(v) => state.add(*v),
            ValueAction::SetValue(v) => state.set(*v as u8),
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

            ValueAction::AddValue(v) => state.add_offset(*v, *offset),
            ValueAction::SetValue(v) => state.set_offset(*v as u8, *offset),
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
            state.set(*v as u8);
            state.move_ptr(*o);
        }

        OptAction::AddAndMove(v, o) => {
            state.add(*v);
            state.move_ptr(*o);
        }

        OptAction::CopyLoop(v) => {
            let cur = *state.tape_ptr as i64;

            for (o, v) in v {
                let ptr = state.tape_ptr.wrapping_offset(*o as isize);
                let val = *ptr as i64;

                *ptr = (val + cur * *v) as u8;
            }

            *state.tape_ptr = 0;
        }

        OptAction::Scan(skip) => {
            while *state.tape_ptr != 0 {
                state.tape_ptr = state.tape_ptr.wrapping_offset(*skip as isize);
            }
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn interpret<W: Write, R: Read>(
    program: &Vec<OptAction>,
    output: &mut W,
    input: &mut R,
) {
    let mut state = ProgramState::new();

    for insn in program {
        eval(insn, output, input, &mut state);
    }
}
