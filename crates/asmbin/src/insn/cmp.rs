use crate::{
    data::RegDataRef,
    insn::{EncodeOpts, InsnEncode, InsnInfo, encode_insn_with},
    reg::Reg,
};

/// Compare [`Self::1`] with [`Self::0`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CmpInsn(pub RegDataRef, pub RegDataRef);

impl CmpInsn {
    pub const fn opcode(&self) -> u8 {
        match self.1 {
            RegDataRef::Direct(reg) | RegDataRef::DirectValue(reg) => {
                if reg.bit_width() == 8 {
                    0x3A
                } else {
                    0x3B
                }
            }

            RegDataRef::RegOffset8(_, _) => 0x3B,
            RegDataRef::RegOffset32(_, _) => 0x3B,

            RegDataRef::Value8(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x3C
                } else {
                    0x80
                }
            }

            RegDataRef::Value16(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x3D
                } else {
                    0x81
                }
            }

            RegDataRef::Value32(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x3D
                } else {
                    0x81
                }
            }

            RegDataRef::Value64(_) => panic!("cmp does not support 64-bit immediate operands!"),
        }
    }
}

impl const InsnInfo for CmpInsn {
    fn predict_size(&self) -> usize {
        let modrm = self.opcode() != 0x3C && self.opcode() != 0x3D;
        let rex = self.0.needs_rex() || self.1.needs_rex();

        self.0.added_bytes() + self.1.added_bytes() + 1 + (modrm as usize) + (rex as usize)
    }
}

impl InsnEncode for CmpInsn {
    fn encode(self) -> Vec<u8> {
        encode_insn_with(EncodeOpts {
            opcode: vec![self.opcode()],
            reg: self.0,
            data: Some(self.1),
            skip_modrm: self.opcode() == 0x3C || self.opcode() == 0x3D,
            invert_operands: false,

            modrm_reg: match self.opcode() {
                0x80 | 0x81 => Some(7),
                _ => None,
            },
        })
    }
}
