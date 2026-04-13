use ristretto_classfile::attributes::Instruction;

use crate::backend::jvm::CodeGenerator;

impl<'a> CodeGenerator<'a> {
    pub(super) fn add_slot(&mut self, amount: i64) {
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.add(Instruction::Dup2);
        self.add(Instruction::Baload);
        self.add(Instruction::Bipush(amount as i8));
        self.add(Instruction::Iadd);
        self.add(Instruction::I2b);
        self.add(Instruction::Bastore);
    }

    pub(super) fn add_slot_offset(&mut self, amount: i64, offset: i64) {
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.ldc(offset);
        self.add(Instruction::Iadd);
        self.add(Instruction::Dup2);
        self.add(Instruction::Baload);
        self.add(Instruction::Bipush(amount as i8));
        self.add(Instruction::Iadd);
        self.add(Instruction::I2b);
        self.add(Instruction::Bastore);
    }

    pub(super) fn set_slot(&mut self, value: i64) {
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.add(Instruction::Bipush(value as i8));
        self.add(Instruction::Bastore);
    }

    pub(super) fn set_slot_offset(&mut self, value: i64, offset: i64) {
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.ldc(offset);
        self.add(Instruction::Iadd);
        self.add(Instruction::Bipush(value as i8));
        self.add(Instruction::Bastore);
    }
}
