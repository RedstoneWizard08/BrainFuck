use ristretto_classfile::attributes::Instruction;

use crate::backend::jvm::CodeGenerator;

impl<'a> CodeGenerator<'a> {
    pub(super) fn print_slot(&mut self) {
        self.add(Instruction::Getstatic(self.id_system_out));
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.add(Instruction::Baload);
        self.add(Instruction::I2c);

        if !self.opts.no_io {
            self.add(Instruction::Invokevirtual(self.id_printstream_append));
        } else {
            self.add(Instruction::Pop);
        }
    }

    pub(super) fn print_slot_offset(&mut self, offset: i64) {
        self.add(Instruction::Getstatic(self.id_system_out));
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.ldc(offset);
        self.add(Instruction::Iadd);
        self.add(Instruction::Baload);
        self.add(Instruction::I2c);

        if !self.opts.no_io {
            self.add(Instruction::Invokevirtual(self.id_printstream_append));
        } else {
            self.add(Instruction::Pop);
        }
    }

    pub(super) fn bulk_print(&mut self, n: i64) {
        self.add(Instruction::Getstatic(self.id_system_out));
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.add(Instruction::Baload);
        self.add(Instruction::I2c);

        if !self.opts.no_io {
            for _ in 0..n {
                self.add(Instruction::Dup);
                self.add(Instruction::Invokevirtual(self.id_printstream_append));
            }

            self.add(Instruction::Pop);
        } else {
            self.add(Instruction::Pop);
        }
    }

    pub(super) fn bulk_print_offset(&mut self, n: i64, offset: i64) {
        self.add(Instruction::Getstatic(self.id_system_out));
        self.add(Instruction::Aload_1);
        self.add(Instruction::Iload_2);
        self.ldc(offset);
        self.add(Instruction::Iadd);
        self.add(Instruction::Baload);
        self.add(Instruction::I2c);

        if !self.opts.no_io {
            for _ in 0..n {
                self.add(Instruction::Dup);
                self.add(Instruction::Invokevirtual(self.id_printstream_append));
            }

            self.add(Instruction::Pop);
        } else {
            self.add(Instruction::Pop);
        }
    }

    pub(super) fn input_slot(&mut self) {
        todo!("JVM backend: stdin");
    }

    pub(super) fn input_slot_offset(&mut self, _offset: i64) {
        todo!("JVM backend: stdin");
    }
}
