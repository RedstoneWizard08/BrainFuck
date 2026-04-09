use crate::{
    data::RegDataRef,
    insn::{EncodeOpts, InsnEncode, InsnInfo, encode_insn_with},
    reg::Reg,
};

/// Add [`Self::1`] to [`Self::0`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AddInsn(pub RegDataRef, pub RegDataRef);

impl AddInsn {
    pub const fn opcode(&self) -> u8 {
        match self.1 {
            RegDataRef::Direct(reg) | RegDataRef::DirectValue(reg) => {
                if reg.bit_width() == 8 {
                    0x00
                } else {
                    0x01
                }
            }

            RegDataRef::RegOffset8(_, _) => {
                if self.0.bit_width() == 8 {
                    0x00
                } else {
                    0x01
                }
            }

            RegDataRef::RegOffset32(_, _) => {
                if self.0.bit_width() == 8 {
                    0x00
                } else {
                    0x01
                }
            }

            RegDataRef::Value8(_) => {
                if matches!(self.0, RegDataRef::Direct(Reg::Rax)) {
                    0x04
                } else if self.0.bit_width() != 8 {
                    0x83
                } else {
                    0x80
                }
            }

            RegDataRef::Value16(_) => {
                if matches!(self.0, RegDataRef::Direct(Reg::Rax)) {
                    0x05
                } else {
                    0x81
                }
            }

            RegDataRef::Value32(_) => {
                if matches!(self.0, RegDataRef::Direct(Reg::Rax)) {
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
        (match self.opcode() {
            0x05 | 0x04 => 1,
            _ => 2,
        }) + self.0.added_bytes()
            + self.1.added_bytes()
            + (self.0.needs_rex() || self.1.needs_rex()) as usize
    }
}

impl InsnEncode for AddInsn {
    fn encode(self) -> Vec<u8> {
        encode_insn_with(EncodeOpts {
            opcode: vec![self.opcode()],
            reg: self.0.into(),
            data: Some(self.1),
            skip_modrm: self.opcode() == 0x05 || self.opcode() == 0x04,
            modrm_reg: None,
            invert_operands: self.1.is_value(),
        })
    }
}
