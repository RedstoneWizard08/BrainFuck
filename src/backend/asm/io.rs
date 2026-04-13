//! I/O operation code generation for the ASM backend.

use crate::backend::asm::CodeGenerator;
use asmbin::{buf::InsnBuf, builders::InsnBuilder, reg::Reg};

impl<'a> CodeGenerator<'a> {
    pub(super) fn print_slot(&mut self, buf: &mut InsnBuf) {
        buf.lea(Reg::Ebx, Reg::Rsi); // load
        buf.mov_to_reg(1_u32, Reg::Eax); // op = sys_write
        buf.mov_to_reg(1_u32, Reg::Edi); // fd = 1 (stdout)
        buf.mov_to_reg(1_u32, Reg::Edx); // len = 1

        if !self.opts.no_io {
            buf.syscall();
        }
    }

    pub(super) fn print_slot_offset(&mut self, buf: &mut InsnBuf, offset: i64) {
        buf.lea(Reg::Ebx + offset, Reg::Rsi); // load
        buf.mov_to_reg(1_u32, Reg::Eax); // op = sys_write
        buf.mov_to_reg(1_u32, Reg::Edi); // fd = 1 (stdout)
        buf.mov_to_reg(1_u32, Reg::Edx); // len = 1

        if !self.opts.no_io {
            buf.syscall();
        }
    }

    pub(super) fn bulk_print(&mut self, buf: &mut InsnBuf, n: i64) {
        buf.lea(Reg::Ebx, Reg::Rsi); // load
        buf.mov_to_reg(1_u32, Reg::Eax); // op = sys_write
        buf.mov_to_reg(1_u32, Reg::Edi); // fd = 1 (stdout)
        buf.mov_to_reg(1_u32, Reg::Edx); // len = 1

        if !self.opts.no_io {
            for _ in 0..n {
                buf.mov_to_reg(1_u32, Reg::Eax); // op = sys_write
                buf.syscall();
            }
        }
    }

    pub(super) fn bulk_print_offset(&mut self, buf: &mut InsnBuf, n: i64, offset: i64) {
        buf.lea(Reg::Ebx + offset, Reg::Rsi); // load
        buf.mov_to_reg(1_u32, Reg::Eax); // op = sys_write
        buf.mov_to_reg(1_u32, Reg::Edi); // fd = 1 (stdout)
        buf.mov_to_reg(1_u32, Reg::Edx); // len = 1

        if !self.opts.no_io {
            for _ in 0..n {
                buf.mov_to_reg(1_u32, Reg::Eax); // op = sys_write
                buf.syscall();
            }
        }
    }

    pub(super) fn input_slot(&mut self, _buf: &mut InsnBuf) {
        todo!("ASM backend: stdin");
    }

    pub(super) fn input_slot_offset(&mut self, _buf: &mut InsnBuf, _offset: i64) {
        todo!("ASM backend: stdin");
    }
}
