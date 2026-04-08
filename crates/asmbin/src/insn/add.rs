use crate::{
    data::RegDataRef,
    insn::{InsnEncode, InsnInfo, encode_insn},
    reg::Reg,
};

/// Add [`Self::1`] to [`Self::0`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddInsn(pub Reg, pub RegDataRef);

impl AddInsn {
    pub const fn opcode(&self) -> u8 {
        match self.1 {
            RegDataRef::Direct(_) | RegDataRef::DirectValue(_) => 0x03, // FIXME: 8-bit might need 0x02
            RegDataRef::RegOffset8(_, _) => 0x03,
            RegDataRef::RegOffset32(_, _) => 0x03,

            RegDataRef::Value8(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x04
                } else {
                    0x80
                }
            }

            RegDataRef::Value16(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x05
                } else {
                    0x81
                }
            }

            RegDataRef::Value32(_) => {
                if self.0.id_bits() == Reg::Rax.id_bits() {
                    0x05
                } else {
                    0x81
                }
            }

            RegDataRef::Value64(_) => panic!("add does not support 64-bit immediate operands!"),
        }
    }
}

impl const InsnInfo for AddInsn {
    fn predict_size(&self) -> usize {
        match self.opcode() {
            0x05 | 0x04 => self.1.added_bytes() + 2,
            _ => self.1.added_bytes() + 3,
        }
    }
}

impl InsnEncode for AddInsn {
    fn encode(self) -> Vec<u8> {
        encode_insn(
            self.opcode(),
            self.0,
            Some(self.1),
            self.opcode() == 0x05 || self.opcode() == 0x04,
        )
    }
}
