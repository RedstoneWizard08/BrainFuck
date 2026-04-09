use crate::{
    data::RegDataRef,
    insn::{InsnEncode, InsnInfo, encode_insn},
    reg::Reg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MovInsn {
    DataToReg(RegDataRef, RegDataRef),
    RegToData(RegDataRef, RegDataRef),
    ByteToReg(RegDataRef, Reg),
}

impl MovInsn {
    pub fn opcode(&self) -> Vec<u8> {
        match self {
            MovInsn::DataToReg(data, reg) => {
                vec![match data {
                    RegDataRef::Direct(reg) | RegDataRef::DirectValue(reg) => {
                        if reg.bit_width() == 8 { 0x8A } else { 0x8B }
                    }

                    RegDataRef::RegOffset8(_, _) => 0x8B,
                    RegDataRef::RegOffset32(_, _) => 0x8B,
                    RegDataRef::Value8(_) => 0xC6,
                    RegDataRef::Value16(_) => 0xC7,
                    RegDataRef::Value32(_) => 0xC7,
                    RegDataRef::Value64(_) => 0xB8 + reg.id_bits(),
                }]
            }

            MovInsn::ByteToReg(_, _) => vec![0x0F, 0xB6],

            MovInsn::RegToData(reg, data) => match reg {
                RegDataRef::Value8(_) => vec![0xC6],
                RegDataRef::Value16(_) | RegDataRef::Value32(_) | RegDataRef::Value64(_) => {
                    vec![0xC7]
                }

                _ => match data {
                    RegDataRef::Value8(_)
                    | RegDataRef::Value16(_)
                    | RegDataRef::Value32(_)
                    | RegDataRef::Value64(_) => panic!(
                        "RegToData is only cannot move data to a constant! That doesn't fucking make sense!"
                    ),

                    _ => vec![0x89],
                },
            },
        }
    }
}

impl const InsnInfo for MovInsn {
    fn predict_size(&self) -> usize {
        match self {
            MovInsn::DataToReg(data, reg) => {
                let rex = data.needs_rex() || reg.needs_rex();
                let modrm = !matches!(data, RegDataRef::Value64(_));

                data.added_bytes() + reg.added_bytes() + 1 + (modrm as usize) + (rex as usize)
            }

            MovInsn::ByteToReg(data, reg) => {
                let rex = data.needs_rex() || reg.needs_rex();
                let modrm = !matches!(data, RegDataRef::Value64(_));

                data.added_bytes() + 2 + (modrm as usize) + (rex as usize)
            }

            MovInsn::RegToData(reg, data) => {
                data.added_bytes()
                    + reg.added_bytes()
                    + 2
                    + ((reg.needs_rex() || data.needs_rex()) as usize)
            }
        }
    }
}

impl InsnEncode for MovInsn {
    fn encode(self) -> Vec<u8> {
        match self {
            MovInsn::DataToReg(data, reg) => {
                let skip_modrm = matches!(data, RegDataRef::Value64(_));

                encode_insn(self.opcode(), reg, Some(data), skip_modrm)
            }

            MovInsn::ByteToReg(data, reg) => {
                let skip_modrm = matches!(data, RegDataRef::Value64(_));

                encode_insn(self.opcode(), reg, Some(data), skip_modrm)
            }

            MovInsn::RegToData(reg, data) => encode_insn(self.opcode(), reg, Some(data), false),
        }
    }
}
