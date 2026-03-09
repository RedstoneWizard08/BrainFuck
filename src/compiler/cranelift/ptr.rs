use crate::{compiler::cranelift::CodeGenerator, interp::wrapping_conv};
use cranelift::prelude::{InstBuilder, MemFlags};
use cranelift_module::Module;

impl<'a, M: Module> CodeGenerator<'a, M> {
    pub(super) fn set_move(&mut self, value: i64, offset: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let post = self.b.ins().iadd_imm(base_addr, offset);
        let value = self.b.ins().iconst(self.byte, wrapping_conv(value) as i64);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
        self.b.def_var(self.tape_ptr, post);
    }

    pub(super) fn add_move(&mut self, amount: i64, offset: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let post = self.b.ins().iadd_imm(base_addr, offset);
        let value = self.b.ins().load(self.byte, MemFlags::new(), base_addr, 0);
        let value = self.b.ins().iadd_imm(value, amount);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
        self.b.def_var(self.tape_ptr, post);
    }

    pub(super) fn move_ptr(&mut self, amount: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let new_addr = self.b.ins().iadd_imm(base_addr, amount);

        self.b.def_var(self.tape_ptr, new_addr);
    }
}
