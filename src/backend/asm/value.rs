use crate::backend::asm::CodeGenerator;
use asmbin::{buf::InsnBuf, builders::InsnBuilder, reg::Reg};

impl<'a> CodeGenerator<'a> {
    pub(super) fn add_slot(&mut self, buf: &mut InsnBuf, amount: i64) {
        if amount == 1 {
            buf.inc([Reg::Rbx]);
        } else if amount == -1 {
            buf.dec([Reg::Rbx]);
        } else {
            if amount > 0 {
                if amount <= u8::MAX as i64 {
                    buf.add([Reg::Rbx], amount as u8);
                } else {
                    buf.add([Reg::Rbx], amount as u32);
                }
            } else {
                let amount = -amount;

                if amount <= u8::MAX as i64 {
                    buf.sub([Reg::Rbx], amount as u8);
                } else {
                    buf.sub([Reg::Rbx], amount as u32);
                }
            }
        }
    }

    pub(super) fn add_slot_offset(&mut self, buf: &mut InsnBuf, amount: i64, offset: i64) {
        if amount == 1 {
            buf.inc(Reg::Rbx + offset);
        } else if amount == -1 {
            buf.dec(Reg::Rbx + offset);
        } else {
            if amount > 0 {
                if amount <= u8::MAX as i64 {
                    buf.add(Reg::Rbx + offset, amount as u8);
                } else {
                    buf.add(Reg::Rbx + offset, amount as u32);
                }
            } else {
                let amount = -amount;

                if amount <= u8::MAX as i64 {
                    buf.sub(Reg::Rbx + offset, amount as u8);
                } else {
                    buf.sub(Reg::Rbx + offset, amount as u32);
                }
            }
        }
    }

    pub(super) fn set_slot(&mut self, buf: &mut InsnBuf, value: i64) {
        buf.mov_from_reg(value as u8, [Reg::Rbx]);
    }

    pub(super) fn set_slot_offset(&mut self, buf: &mut InsnBuf, value: i64, offset: i64) {
        buf.mov_from_reg(value as u8, Reg::Rbx + offset);
    }
}
