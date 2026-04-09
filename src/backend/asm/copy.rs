use crate::backend::asm::CodeGenerator;
use asmbin::{buf::InsnBuf, builders::InsnBuilder, reg::Reg};

impl<'a> CodeGenerator<'a> {
    pub(super) fn copy_loop(&mut self, buf: &mut InsnBuf, values: &Vec<(i64, i64)>) {
        buf.mov_byte_to_reg([Reg::Rbx], Reg::Eax);

        for (offset, mul) in values {
            if *mul == 1 {
                buf.add(Reg::Al, Reg::Rbx + *offset);
            } else {
                if *mul <= u8::MAX as i64 {
                    buf.imul_imm(Reg::Ecx, Reg::Eax, *mul as u8);
                } else {
                    buf.imul_imm(Reg::Ecx, Reg::Eax, *mul as u32);
                }

                buf.add(Reg::Cl, Reg::Rbx + *offset);
            }
        }

        buf.mov_from_reg(0_u8, [Reg::Rbx]);
    }
}
