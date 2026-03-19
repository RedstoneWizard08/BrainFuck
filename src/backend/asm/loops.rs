use crate::{
    backend::asm::{
        CodeGenerator,
        insn::{AsmBuilder, Reg},
    },
    opt::OptAction,
};

impl<'a> CodeGenerator<'a> {
    pub(super) fn translate_loop(&mut self, actions: &Vec<OptAction>) {
        let name = format!("br_{}", self.block);
        let end = format!("br_end_{}", self.block);

        self.block += 1;

        self.label(&name);
        self.mov(Reg::Eax, self.ptr.ptr());
        self.cmp(Reg::Al, 0);
        self.je(&end);

        for insn in actions {
            self.translate(insn);
        }

        self.jmp(name);
        self.label(end);
    }
}
