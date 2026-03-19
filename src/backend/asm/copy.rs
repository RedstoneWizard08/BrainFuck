use std::collections::BTreeMap;

use crate::backend::asm::{
    CodeGenerator,
    insn::{AsmBuilder, Reg},
};

impl<'a> CodeGenerator<'a> {
    pub(super) fn copy_loop(&mut self, values: &BTreeMap<i64, i64>) {
        self.mov(Reg::Eax, self.ptr.ptr());

        for (offset, mul) in values {
            if *mul == 1 {
                self.add(self.ptr.ptr_offs(*offset), Reg::Al);
            } else {
                self.imul(Reg::Ecx, Reg::Eax, *mul);
                self.add(self.ptr.ptr_offs(*offset), Reg::Cl);
            }
        }

        self.mov(self.ptr.ptr(), 0);
    }
}
