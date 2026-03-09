use crate::compiler::cranelift::CodeGenerator;
use cranelift::prelude::{InstBuilder, MemFlags, Value};
use cranelift_module::Module;

impl<'a, M: Module> CodeGenerator<'a, M> {
    pub(super) fn write_to_arr(&mut self, value: Value) {
        let base_addr = self.b.use_var(self.tape_ptr);

        self.b.ins().store(MemFlags::new(), value, base_addr, 0);
    }

    pub(super) fn read_from_arr(&mut self) -> Value {
        let base_addr = self.b.use_var(self.tape_ptr);

        self.b.ins().load(self.byte, MemFlags::new(), base_addr, 0)
    }

    pub(super) fn write_to_arr_offset(&mut self, value: Value, offset: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let addr = self.b.ins().iadd_imm(base_addr, offset);

        self.b.ins().store(MemFlags::new(), value, addr, 0);
    }

    pub(super) fn read_from_arr_offset(&mut self, offset: i64) -> Value {
        let base_addr = self.b.use_var(self.tape_ptr);
        let addr = self.b.ins().iadd_imm(base_addr, offset);

        self.b.ins().load(self.byte, MemFlags::new(), addr, 0)
    }
}
