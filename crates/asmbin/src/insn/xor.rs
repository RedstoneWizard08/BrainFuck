use crate::{
    data::RegDataRef,
    insn::{EncodeOpts, InsnEncode, InsnInfo, encode_insn_with},
    reg::Reg,
};

/// XOR [`Self::1`] with [`Self::0`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct XorInsn(pub Reg, pub RegDataRef);

impl XorInsn {
    pub const fn opcode(&self) -> u8 {
        match self.1 {
            RegDataRef::Direct(_) | RegDataRef::DirectValue(_) => 0x33, // FIXME: 8-bit might need 0x32
            RegDataRef::RegOffset8(_, _) => 0x33,
            RegDataRef::RegOffset32(_, _) => 0x33,

            RegDataRef::Value8(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x34
                } else {
                    0x80
                }
            }

            RegDataRef::Value16(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x35
                } else {
                    0x81
                }
            }

            RegDataRef::Value32(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x35
                } else {
                    0x81
                }
            }

            RegDataRef::Value64(_) => panic!("cmp does not support 64-bit immediate operands!"),
        }
    }
}

impl const InsnInfo for XorInsn {
    fn predict_size(&self) -> usize {
        self.1.added_bytes() + 3
    }
}

impl InsnEncode for XorInsn {
    fn encode(self) -> Vec<u8> {
        encode_insn_with(EncodeOpts {
            opcode: self.opcode(),
            reg: self.0,
            data: Some(self.1),
            skip_modrm: self.opcode() == 0x34 || self.opcode() == 0x35,

            modrm_reg: match self.opcode() {
                0x80 | 0x81 => Some(6),
                _ => None,
            },
        })
    }
}
