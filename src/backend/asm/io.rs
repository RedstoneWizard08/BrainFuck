use crate::backend::asm::{
    CodeGenerator,
    insn::{AsmBuilder, Reg},
};

impl<'a> CodeGenerator<'a> {
    pub(super) fn print_slot(&mut self) {
        self.lea(Reg::Rsi, self.ptr.ptr()); // load
        self.mov(Reg::Eax, 1); // op = sys_write
        self.mov(Reg::Edi, 1); // fd = 1 (stdout)
        self.mov(Reg::Edx, 1); // len = 1

        if !self.opts.no_io {
            self.syscall();
        }
    }

    pub(super) fn print_slot_offset(&mut self, offset: i64) {
        self.lea(Reg::Rsi, self.ptr.ptr_offs(offset)); // load
        self.mov(Reg::Eax, 1); // op = sys_write
        self.mov(Reg::Edi, 1); // fd = 1 (stdout)
        self.mov(Reg::Edx, 1); // len = 1

        if !self.opts.no_io {
            self.syscall();
        }
    }

    pub(super) fn bulk_print(&mut self, n: i64) {
        self.lea(Reg::Rsi, self.ptr.ptr()); // load
        self.mov(Reg::Eax, 1); // op = sys_write
        self.mov(Reg::Edi, 1); // fd = 1 (stdout)
        self.mov(Reg::Edx, 1); // len = 1

        if !self.opts.no_io {
            for _ in 0..n {
                self.mov(Reg::Eax, 1); // op = sys_write
                self.syscall();
            }
        }
    }

    pub(super) fn bulk_print_offset(&mut self, n: i64, offset: i64) {
        self.lea(Reg::Rsi, self.ptr.ptr_offs(offset)); // load
        self.mov(Reg::Eax, 1); // op = sys_write
        self.mov(Reg::Edi, 1); // fd = 1 (stdout)
        self.mov(Reg::Edx, 1); // len = 1

        if !self.opts.no_io {
            for _ in 0..n {
                self.mov(Reg::Eax, 1); // op = sys_write
                self.syscall();
            }
        }
    }

    pub(super) fn input_slot(&mut self) {
        todo!("ASM backend: stdin");
    }

    pub(super) fn input_slot_offset(&mut self, _offset: i64) {
        todo!("ASM backend: stdin");
    }
}
