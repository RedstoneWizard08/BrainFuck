use crate::{
    backend::asm::{CodeGenerator, insn::AsmBuilder},
    opt::OptAction,
};

impl<'a> CodeGenerator<'a> {
    pub(super) fn translate_loop(&mut self, actions: &Vec<OptAction>) {
        let name = format!("br_{}", self.block);
        let end = format!("br_end_{}", self.block);

        self.block += 1;

        self.label(&name);
        self.cmp(self.ptr.ptr(), 0);
        self.je(&end);

        for insn in actions {
            self.translate(insn);
        }

        self.jmp(name);
        self.label(end);
    }

    pub(super) fn scan(&mut self, skip: i64) {
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
