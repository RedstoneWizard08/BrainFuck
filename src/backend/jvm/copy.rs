use crate::backend::jvm::CodeGenerator;
use ristretto_classfile::attributes::Instruction;

impl<'a> CodeGenerator<'a> {
    pub(super) fn copy_loop(&mut self, values: &Vec<(i64, i64)>) {
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);

        for (offset, mul) in values {
            if *mul == 1 {
                self.add(Instruction::Dup2);
                self.add(Instruction::Dup2);
                self.ldc(*offset);
                self.add(Instruction::Iadd);
                self.add(Instruction::Baload);
                self.add(Instruction::Baload);
                self.add(Instruction::Iadd);
                self.add(Instruction::I2b);
                self.add(Instruction::Bastore);
            } else {
                self.add(Instruction::Dup2);
                self.add(Instruction::Dup2);
                self.ldc(*offset);
                self.add(Instruction::Iadd);
                self.add(Instruction::Baload);
                self.add(Instruction::Baload);
                self.ldc(*mul);
                self.add(Instruction::Imul);
                self.add(Instruction::Iadd);
                self.add(Instruction::I2b);
                self.add(Instruction::Bastore);
            }
        }

        self.add(Instruction::Iconst_0);
        self.add(Instruction::Bastore);
    }
}
