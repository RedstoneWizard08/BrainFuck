use asmbin::{
    buf::InsnBuf,
    builders::{InsnBuilder, InsnRecv},
    insn::jmp::JmpCond,
    reg::Reg,
};

use crate::{backend::asm::CodeGenerator, opt::OptAction};

impl<'a> CodeGenerator<'a> {
    pub(super) fn translate_loop(&mut self, buf: &mut InsnBuf, actions: &Vec<OptAction>) {
        if self.known_zero {
            return;
        }

        let mut tmp = InsnBuf::new();

        for insn in actions {
            self.translate(&mut tmp, insn);
        }

        let mut end = InsnBuf::new();

        end.cmp([Reg::Rbx], 0u8);

        let jmp_dist = tmp.calculate_length() + end.calculate_length();

        // jmp_cond_rel32 is 6 bytes
        end.jmp_cond_rel32(JmpCond::NotEqual, -(jmp_dist as i32) - 6);

        if !self.known_nonzero {
            buf.jmp_rel32(tmp.calculate_length() as i32);
        }

        buf.extend(tmp);
        buf.extend(end);
    }

    pub(super) fn scan(&mut self, buf: &mut InsnBuf, skip: i64) {
        // TODO: skip == 1 -> repne scasb

        let mut tmp = InsnBuf::new();

        if skip >= 0 {
            tmp.add(Reg::Rbx, skip as u32);
        } else {
            tmp.sub(Reg::Rbx, (-skip) as u32);
        }

        tmp.cmp([Reg::Rbx], 0u8);

        let buf_len = tmp.calculate_length();

        if buf_len <= (u8::MAX - 3) as u64 {
            // jmp_cond_rel8 is 3 bytes
            tmp.jmp_cond_rel8(JmpCond::NotEqual, -(buf_len as i8) - 3);
        } else {
            // jmp_cond_rel32 is 6 bytes
            tmp.jmp_cond_rel32(JmpCond::NotEqual, -(buf_len as i32) - 6);
        }

        let buf_len = tmp.calculate_length();

        buf.cmp([Reg::Rbx], 0u8);

        if buf_len <= u8::MAX as u64 {
            buf.jmp_cond_rel8(JmpCond::Equal, buf_len as i8);
        } else {
            buf.jmp_cond_rel32(JmpCond::Equal, buf_len as i32);
        }

        buf.extend(tmp);
    }
}
