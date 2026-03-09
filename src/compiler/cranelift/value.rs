use crate::{compiler::cranelift::CodeGenerator, interp::wrapping_conv};
use cranelift::prelude::{InstBuilder, MemFlags};
use cranelift_module::Module;

impl<'a, M: Module> CodeGenerator<'a, M> {
    pub(super) fn add_slot(&mut self, amount: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let value = self.b.ins().load(self.byte, MemFlags::new(), base_addr, 0);
        let value = self.b.ins().iadd_imm(value, amount);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    pub(super) fn set_slot(&mut self, value: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let value = self.b.ins().iconst(self.byte, wrapping_conv(value) as i64);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    pub(super) fn add_slot_offset(&mut self, amount: i64, offset: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let addr = self.b.ins().iadd_imm(base_addr, offset);
        let value = self.b.ins().load(self.byte, MemFlags::new(), addr, 0);
        let value = self.b.ins().iadd_imm(value, amount);

        self.b.ins().store(MemFlags::new(), value, addr, 0);
    }

    pub(super) fn set_slot_offset(&mut self, value: i64, offset: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let addr = self.b.ins().iadd_imm(base_addr, offset);
        let value = self.b.ins().iconst(self.byte, wrapping_conv(value) as i64);

        self.b.ins().store(MemFlags::new(), value, addr, 0);
    }
}
