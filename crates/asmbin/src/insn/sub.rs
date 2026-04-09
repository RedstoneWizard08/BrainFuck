use crate::{
    data::RegDataRef,
    insn::{EncodeOpts, InsnEncode, InsnInfo, encode_insn_with},
    reg::Reg,
};

/// Subtract [`Self::1`] from [`Self::0`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubInsn(pub RegDataRef, pub RegDataRef);

impl SubInsn {
    pub const fn opcode(&self) -> u8 {
        match self.1 {
            RegDataRef::Direct(reg) | RegDataRef::DirectValue(reg) => {
                if reg.bit_width() == 8 {
                    0x28
                } else {
                    0x29
                }
            }

            RegDataRef::RegOffset8(_, _) => {
                if self.0.bit_width() == 8 {
                    0x28
                } else {
                    0x29
                }
            }

            RegDataRef::RegOffset32(_, _) => {
                if self.0.bit_width() == 8 {
                    0x28
                } else {
                    0x29
                }
            }

            RegDataRef::Value8(_) => {
                if matches!(self.0, RegDataRef::Direct(Reg::Rax)) {
                    0x2C
                } else if self.0.bit_width() != 8 {
                    // 0x83 /5; MODRM: reg = 5, rm = target_reg
                    0x83
                } else {
                    // 0x80 /5; MODRM: reg = 5, rm = target_reg
                    0x80
                }
            }

            RegDataRef::Value16(_) => {
                if matches!(self.0, RegDataRef::Direct(Reg::Rax)) {
                    0x2D
                } else {
                    // 0x81 /5; MODRM: reg = 5, rm = target_reg
                    0x81
                }
            }

            RegDataRef::Value32(_) => {
                if matches!(self.0, RegDataRef::Direct(Reg::Rax)) {
                    0x2D
                } else {
                    // 0x81 /5; MODRM: reg = 5, rm = target_reg
                    0x81
                }
            }

            RegDataRef::Value64(_) => panic!("sub does not support 64-bit immediate operands!"),
        }
    }
}

impl const InsnInfo for SubInsn {
    fn predict_size(&self) -> usize {
        (match self.opcode() {
            0x2C | 0x2D => 1,
            _ => 2,
        }) + self.0.added_bytes()
            + self.1.added_bytes()
            + (self.0.needs_rex() || self.1.needs_rex()) as usize
    }
}

impl InsnEncode for SubInsn {
    fn encode(self) -> Vec<u8> {
        encode_insn_with(EncodeOpts {
            opcode: vec![self.opcode()],
            reg: self.0,
            data: Some(self.1),
            skip_modrm: self.opcode() == 0x2C || self.opcode() == 0x2D,
            invert_operands: self.1.is_value(),

            modrm_reg: match self.opcode() {
                0x80 | 0x81 | 0x83 => Some(5),
                _ => None,
            },
        })
    }
}
