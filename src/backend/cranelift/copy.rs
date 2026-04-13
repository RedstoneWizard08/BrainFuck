//! Copy loop code generation for the Cranelift backend.

use crate::backend::cranelift::CodeGenerator;
use cranelift::prelude::{InstBuilder, MemFlags};
use cranelift_module::Module;

impl<'a, M: Module> CodeGenerator<'a, M> {
    pub(super) fn copy_loop(&mut self, values: &Vec<(i64, i64)>) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let value = self.b.ins().load(self.byte, MemFlags::new(), base_addr, 0);

        for (offset, mul) in values {
            let addr = self.b.ins().iadd_imm(base_addr, *offset);
            let cur = self.b.ins().load(self.byte, MemFlags::new(), addr, 0);
            let additional = self.b.ins().imul_imm(value, *mul);
            let result = self.b.ins().iadd(cur, additional);

            self.b.ins().store(MemFlags::new(), result, addr, 0);
        }

        let zero = self.b.ins().iconst(self.byte, 0);

        self.b.ins().store(MemFlags::new(), zero, base_addr, 0);
    }
}
