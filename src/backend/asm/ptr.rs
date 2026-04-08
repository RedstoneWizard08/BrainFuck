use crate::backend::asm::{CodeGenerator, insn::AsmBuilder};

impl<'a> CodeGenerator<'a> {
    pub(super) fn set_move(&mut self, value: i64, offset: i64) {
        self.mov(self.ptr.ptr(), value);
        self.move_ptr(offset);
    }

    pub(super) fn add_move(&mut self, amount: i64, offset: i64) {
        self.add_slot(amount);
        self.move_ptr(offset);
    }

    pub(super) fn move_ptr(&mut self, amount: i64) {
        if amount == 1 {
            self.inc(self.ptr);
        } else {
            self.add(self.ptr, amount);
        }
    }
}
