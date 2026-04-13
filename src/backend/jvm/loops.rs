use ristretto_classfile::attributes::Instruction;

use crate::{backend::jvm::CodeGenerator, opt::action::OptAction};

impl<'a> CodeGenerator<'a> {
    pub(super) fn translate_loop(&mut self, actions: &Vec<OptAction>) {
        if self.known_zero {
            return;
        }

        // TODO: Don't assume it's non-zero!

        let pos = self.pos;

        for insn in actions {
            self.translate(insn);
        }

        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.add(Instruction::Baload);
        self.add(Instruction::Iconst_0);

        let dist = self.pos - pos;
        let dist: isize = dist as isize;
        let dist: i16 = -(dist as i16);

        // DOES NOT WORK BECAUSE RISTRETTO CLASSFILE IS BROKEN!
        self.add(Instruction::If_icmpne(dist as u16));
    }

    pub(super) fn scan(&mut self, skip: i64) {
        // TODO: Don't assume it's non-zero!

        let pos = self.pos;

        self.add(Instruction::Iload_2);
        self.ldc(skip);
        self.add(Instruction::Iadd);
        self.add(Instruction::Istore_2);

        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.add(Instruction::Baload);
        self.add(Instruction::Iconst_0);

        let dist = self.pos - pos;
        let dist: isize = dist as isize;
        let dist: i16 = -(dist as i16);

        // DOES NOT WORK BECAUSE RISTRETTO CLASSFILE IS BROKEN!
        self.add(Instruction::If_icmpne(dist as u16));
    }
}
