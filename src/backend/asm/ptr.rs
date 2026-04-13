//! Pointer manipulation code generation for the ASM backend.

use crate::backend::asm::CodeGenerator;
use asmbin::{buf::InsnBuf, builders::InsnBuilder, reg::Reg};

impl<'a> CodeGenerator<'a> {
    pub(super) fn set_move(&mut self, buf: &mut InsnBuf, value: i64, offset: i64) {
        self.set_slot(buf, value);
        self.move_ptr(buf, offset);
    }

    pub(super) fn add_move(&mut self, buf: &mut InsnBuf, amount: i64, offset: i64) {
        self.add_slot(buf, amount);
        self.move_ptr(buf, offset);
    }

    pub(super) fn move_ptr(&mut self, buf: &mut InsnBuf, amount: i64) {
        if amount == 1 {
            buf.inc(Reg::Rbx);
        } else if amount == -1 {
            buf.dec(Reg::Rbx);
        } else if amount >= 0 {
            if amount <= u8::MAX as i64 {
                buf.add(Reg::Rbx, amount as u8);
            } else {
                buf.add(Reg::Rbx, amount as u32);
            }
        } else {
            let amount = -amount;

            if amount <= u8::MAX as i64 {
                buf.sub(Reg::Rbx, amount as u8);
            } else {
                buf.sub(Reg::Rbx, amount as u32);
            }
        }
    }
}
