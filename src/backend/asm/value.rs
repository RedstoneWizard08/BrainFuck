use crate::backend::asm::{CodeGenerator, insn::AsmBuilder};

impl<'a> CodeGenerator<'a> {
    pub(super) fn add_slot(&mut self, amount: i64) {
        if amount == 1 {
            self.inc(self.ptr.ptr());
        } else {
            self.add(self.ptr.ptr(), amount);
        }
    }

    pub(super) fn add_slot_offset(&mut self, amount: i64, offset: i64) {
        self.add(self.ptr.ptr_offs(offset), amount);
    }

    pub(super) fn set_slot(&mut self, value: i64) {
        self.mov(self.ptr.ptr(), value);
    }

    pub(super) fn set_slot_offset(&mut self, value: i64, offset: i64) {
        self.mov(self.ptr.ptr_offs(offset), value);
    }
}
