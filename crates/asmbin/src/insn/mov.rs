use crate::{
    data::RegDataRef,
    insn::{InsnEncode, InsnInfo, encode_insn},
    reg::Reg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MovInsn {
    DataToReg(RegDataRef, Reg),
    RegToData(Reg, RegDataRef),
}

impl MovInsn {
    pub fn opcode(&self) -> u8 {
        match self {
            MovInsn::DataToReg(data, reg) => match data {
                RegDataRef::Direct(_) => 0x8B, // TODO: 8-bit might need 0x8A
                RegDataRef::RegOffset8(_, _) => 0x8B,
                RegDataRef::RegOffset32(_, _) => 0x8B,
                RegDataRef::Value8(_) => 0xC6,
                RegDataRef::Value16(_) => 0xC7,
                RegDataRef::Value32(_) => 0xC7,
                RegDataRef::Value64(_) => {
                    if reg.is_ext() {
                        panic!("movabs doesn't work for extended registers!")
                    } else {
                        0xB8 + reg.id_bits()
                    }
                }
            },

            MovInsn::RegToData(_, data) => match data {
                RegDataRef::Value8(_)
                | RegDataRef::Value16(_)
                | RegDataRef::Value32(_)
                | RegDataRef::Value64(_) => panic!(
                    "RegToData is only cannot move data to a constant! That doesn't fucking make sense!"
                ),

                _ => 0x89,
            },
        }
    }
}

impl const InsnInfo for MovInsn {
    fn predict_size(&self) -> usize {
        match self {
            MovInsn::DataToReg(b, _) => {
                if matches!(b, RegDataRef::Value64(_)) {
                    b.added_bytes() + 2
                } else {
                    b.added_bytes() + 3
                }
            }

            MovInsn::RegToData(_, b) => b.added_bytes() + 3,
        }
    }
}

impl InsnEncode for MovInsn {
    fn encode(self) -> Vec<u8> {
        match self {
            MovInsn::DataToReg(b, a) => {
                let skip_modrm = matches!(b, RegDataRef::Value64(_));

                encode_insn(self.opcode(), a, b, skip_modrm)
            }

            MovInsn::RegToData(a, b) => encode_insn(self.opcode(), a, b, false),
        }
    }
}
