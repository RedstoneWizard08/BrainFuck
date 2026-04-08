use crate::{
    data::RegDataRef,
    insn::{EncodeOpts, InsnEncode, InsnInfo, encode_insn_with},
    reg::Reg,
};

/// Subtract [`Self::1`] from [`Self::0`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubInsn(pub Reg, pub RegDataRef);

impl SubInsn {
    pub const fn opcode(&self) -> u8 {
        match self.1 {
            RegDataRef::Direct(_) | RegDataRef::DirectValue(_) => 0x2B,
            RegDataRef::RegOffset8(_, _) => 0x2B,
            RegDataRef::RegOffset32(_, _) => 0x2B,

            RegDataRef::Value8(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x2C
                } else {
                    // 0x80 /5; MODRM: reg = 5, rm = target_reg
                    0x80
                }
            }

            RegDataRef::Value16(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x2D
                } else {
                    // 0x81 /5; MODRM: reg = 5, rm = target_reg
                    0x81
                }
            }

            RegDataRef::Value32(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
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
        self.1.added_bytes() + 3
    }
}

impl InsnEncode for SubInsn {
    fn encode(self) -> Vec<u8> {
        encode_insn_with(EncodeOpts {
            opcode: self.opcode(),
            reg: self.0,
            data: Some(self.1),
            skip_modrm: self.opcode() == 0x2C || self.opcode() == 0x2D,

            modrm_reg: match self.opcode() {
                0x80 | 0x81 => Some(5),
                _ => None,
            },
        })
    }
}
