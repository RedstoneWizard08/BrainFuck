use crate::{
    any_needs_64,
    data::RegDataRef,
    insn::{InsnEncode, InsnInfo, ModRm, encode_rex, modrm},
    reg::Reg,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImulInsn {
    /// Effectively `*=` for the register you're multiplying.
    Registers {
        /// The register to multiply.
        reg: Reg,

        /// The register containing the multiplier.
        mul: RegDataRef,
    },

    Immediate {
        dst: Reg,
        src: RegDataRef,
        mul: RegDataRef,
    },
}

impl ImulInsn {
    pub fn opcode(&self) -> Vec<u8> {
        match self {
            Self::Registers { .. } => vec![0x0F, 0xAF],

            Self::Immediate { mul, .. } => match *mul {
                RegDataRef::Value8(_) => vec![0x6B],
                RegDataRef::Value16(_) => vec![0x69],
                RegDataRef::Value32(_) => vec![0x69],
                RegDataRef::Value64(_) => panic!("mul does not support 64-bit immediate operands!"),

                _ => panic!("mul in immediate mode does not support register operands!"),
            },
        }
    }
}

impl const InsnInfo for ImulInsn {
    fn predict_size(&self) -> usize {
        match self {
            Self::Registers { reg, mul } => {
                if any_needs_64!(reg, mul) {
                    4
                } else {
                    3
                }
            }

            Self::Immediate { dst, src, mul } => {
                let extra = any_needs_64!(dst, src, mul) as u8 as usize;

                2 + extra + mul.added_bytes()
            }
        }
    }
}

impl InsnEncode for ImulInsn {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            ImulInsn::Registers { reg, mul } => {
                if mul.is_value() {
                    panic!("mul in register mode cannot support value operands!");
                }

                if any_needs_64!(reg, mul) {
                    buf.push(encode_rex(&Some(reg), &Some(mul)));
                }

                buf.extend(self.opcode());

                buf.push(
                    ModRm {
                        mod_: modrm(Some(mul)),
                        reg: reg.id_bits(),
                        rm: mul.id_bits(),
                    }
                    .encode(),
                );

                buf.extend(mul.extra_bytes());
            }

            ImulInsn::Immediate { dst, src, mul } => {
                if src.is_value() {
                    panic!("mul in immediate mode cannot support a value source!");
                }

                if any_needs_64!(dst, src, mul) {
                    buf.push(encode_rex(&Some(dst), &Some(src)));
                }

                buf.extend(self.opcode());

                buf.push(
                    ModRm {
                        mod_: modrm(Some(src)),
                        reg: dst.id_bits(),
                        rm: src.id_bits(),
                    }
                    .encode(),
                );

                buf.extend(src.extra_bytes());
                buf.extend(mul.extra_bytes());
            }
        }

        buf
    }
}
