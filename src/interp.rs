use std::io::{Read, Write};

use crate::{TAPE_SIZE, optimizer::OptAction};

pub struct ProgramState {
    tape: [u8; TAPE_SIZE],
    tape_ptr: usize,
}

impl ProgramState {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            tape: [0; TAPE_SIZE],
            tape_ptr: 0,
        }
    }

    #[inline(always)]
    pub fn add(&mut self, amnt: usize) {
        self.tape[self.tape_ptr] =
            self.tape[self.tape_ptr].wrapping_add((amnt % u8::MAX as usize) as u8);
    }

    #[inline(always)]
    pub fn sub(&mut self, amnt: usize) {
        self.tape[self.tape_ptr] =
            self.tape[self.tape_ptr].wrapping_sub((amnt % u8::MAX as usize) as u8);
    }

    #[inline(always)]
    pub fn right(&mut self, amnt: usize) {
        if self.tape_ptr + amnt >= TAPE_SIZE {
            self.tape_ptr = self.tape_ptr + amnt - TAPE_SIZE;
        } else {
            self.tape_ptr += amnt;
        }
    }

    #[inline(always)]
    pub fn left(&mut self, amnt: usize) {
        if self.tape_ptr <= 0 {
            self.tape_ptr = TAPE_SIZE - amnt;
        } else {
            self.tape_ptr -= amnt;
        }
    }

    #[inline(always)]
    pub fn get(&self) -> u8 {
        self.tape[self.tape_ptr]
    }

    #[inline(always)]
    pub fn set(&mut self, value: u8) {
        self.tape[self.tape_ptr] = value;
    }
}

pub fn eval<W: Write, R: Read>(
    insn: &OptAction,
    output: &mut W,
    input: &mut R,
    state: &mut ProgramState,
) {
    match insn {
        OptAction::Right => state.right(1),
        OptAction::Left => state.left(1),
        OptAction::Inc => state.add(1),
        OptAction::Dec => state.sub(1),

        OptAction::Output => {
            output.write(&[state.get()]).unwrap();
        }

        OptAction::Input => {
            let mut buf = [0u8; 1];

            input.read(&mut buf).unwrap();
            state.set(buf[0]);
        }

        OptAction::Loop(actions) => loop {
            if state.get() == 0 {
                break;
            }

            for insn in actions {
                eval(insn, output, input, state);
            }
        },

        OptAction::Noop => (),
        OptAction::AddValue(v) => state.add(*v),
        OptAction::SubValue(v) => state.sub(*v),
        OptAction::SetValue(v) => state.set((*v % u8::MAX as usize) as u8),
        OptAction::MoveRight(v) => state.right(*v),
        OptAction::MoveLeft(v) => state.left(*v),
        OptAction::ZeroRight(_v) => todo!(),
    }
}

pub fn interpret<W: Write, R: Read>(program: &Vec<OptAction>, output: &mut W, input: &mut R) {
    let mut state = ProgramState::new();

    for insn in program {
        eval(insn, output, input, &mut state);
    }
}
