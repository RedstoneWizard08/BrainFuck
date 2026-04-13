use ristretto_classfile::attributes::Instruction;

use crate::backend::jvm::CodeGenerator;

impl<'a> CodeGenerator<'a> {
    pub(super) fn set_move(&mut self, value: i64, offset: i64) {
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.add(Instruction::Bipush(value as i8));
        self.add(Instruction::Bastore);

        self.move_ptr(offset);
    }

    pub(super) fn add_move(&mut self, amount: i64, offset: i64) {
        self.add_slot(amount);
        self.move_ptr(offset);
    }

    pub(super) fn move_ptr(&mut self, amount: i64) {
        if (amount < i8::MAX as i64) && (amount > i8::MIN as i64) {
            self.add(Instruction::Iinc(2, amount as i8));
        } else {
            self.add(Instruction::Iload_2);
            self.ldc(amount);
            self.add(Instruction::Iadd);
            self.add(Instruction::Istore_2);
        }
    }
}
