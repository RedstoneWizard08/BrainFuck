use crate::backend::asm::{CodeGenerator, insn::AsmBuilder};

impl<'a> CodeGenerator<'a> {
    pub(super) fn set_move(&mut self, value: i64, offset: i64) {
        self.mov(self.ptr.ptr(), value);
        self.add(self.ptr, offset);
    }

    pub(super) fn add_move(&mut self, amount: i64, offset: i64) {
        self.add(self.ptr.ptr(), amount);
        self.add(self.ptr, offset);
    }

    pub(super) fn move_ptr(&mut self, amount: i64) {
        self.add(self.ptr, amount);
    }
}
