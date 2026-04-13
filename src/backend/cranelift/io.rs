//! I/O operation code generation for the Cranelift backend.

use crate::backend::cranelift::CodeGenerator;
use cranelift::prelude::{InstBuilder, types};
use cranelift_module::Module;

impl<'a, M: Module> CodeGenerator<'a, M> {
    pub(super) fn print_slot(&mut self) {
        let value = self.read_from_arr();
        let value = self.b.ins().uextend(types::I32, value);

        self.b.ins().call(self.putchar, &[value]);
    }

    pub(super) fn bulk_print(&mut self, n: i64) {
        let value = self.read_from_arr();
        let value = self.b.ins().uextend(types::I32, value);

        for _ in 0..n {
            self.b.ins().call(self.putchar, &[value]);
        }
    }

    pub(super) fn input_slot(&mut self) {
        let call = self.b.ins().call(self.getchar, &[]);
        let value = self.b.inst_results(call)[0];
        let value = self.b.ins().ireduce(self.byte, value);

        self.write_to_arr(value);
    }

    pub(super) fn print_slot_offset(&mut self, offset: i64) {
        let value = self.read_from_arr_offset(offset);
        let value = self.b.ins().uextend(types::I32, value);

        self.b.ins().call(self.putchar, &[value]);
    }

    pub(super) fn bulk_print_offset(&mut self, n: i64, offset: i64) {
        let value = self.read_from_arr_offset(offset);
        let value = self.b.ins().uextend(types::I32, value);

        for _ in 0..n {
            self.b.ins().call(self.putchar, &[value]);
        }
    }

    pub(super) fn input_slot_offset(&mut self, offset: i64) {
        let call = self.b.ins().call(self.getchar, &[]);
        let value = self.b.inst_results(call)[0];
        let value = self.b.ins().ireduce(self.byte, value);

        self.write_to_arr_offset(value, offset);
    }
}
