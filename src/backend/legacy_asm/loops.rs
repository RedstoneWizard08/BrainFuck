use crate::{
    backend::legacy_asm::{
        CodeGenerator,
        insn::{AsmBuilder, Reg},
    },
    opt::action::OptAction,
};

impl<'a> CodeGenerator<'a> {
    pub(super) fn translate_loop(&mut self, actions: &Vec<OptAction>) {
        if self.known_zero {
            return;
        }

        let name = format!("br_{}", self.block);
        let end = format!("br_end_{}", self.block);

        self.block += 1;

        if !self.known_nonzero {
            self.jmp(&end);
        }

        self.label(&name);

        for insn in actions {
            self.translate(insn);
        }

        self.label(end);
        self.cmp(self.ptr.ptr(), 0);
        self.jne(&name);
    }

    pub(super) fn scan(&mut self, skip: i64) {
        if skip == 1 {
            self.mov(Reg::Rdi.ptr(), self.ptr.ptr());
            self.inc(Reg::Rdi.ptr());
            self.xor(Reg::Eax.ptr(), Reg::Eax.ptr());
            self.mov(Reg::Ecx.ptr(), 65536);
            self.scanbyte();
            self.lea(self.ptr.ptr(), Reg::Rdi.ptr_offs(-1));
        } else {
            let scan = format!("scan_{}", self.block);
            let end = format!("scan_end_{}", self.block);

            self.block += 1;

            self.cmp(self.ptr.ptr(), 0);
            self.je(&end);

            self.label(&scan);
            self.add(self.ptr, skip);
            self.cmp(self.ptr.ptr(), 0);
            self.jne(scan);

            self.label(end);
        }
    }
}
