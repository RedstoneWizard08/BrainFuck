use crate::{
    data::RegDataRef,
    insn::{InsnEncode, InsnInfo, ModRm, modrm},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JmpInsn {
    Short(i8),            // relative jump, signed 1-byte offset
    Near(i32),            // relative jump, signed 4-byte offset
    Indirect(RegDataRef), // absolute jump to arg (rip = arg)
    Cond8(JmpCond, i8),   // absolute conditional jump to a 1-byte offset into the text section
    Cond32(JmpCond, i32), // absolute conditional jump to a 4-byte offset into the text section
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JmpCond {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl JmpCond {
    pub fn rel8(&self) -> Vec<u8> {
        match self {
            JmpCond::Equal => vec![0x74],
            JmpCond::NotEqual => vec![0x75],
            JmpCond::Less => vec![0x7C],
            JmpCond::LessEqual => vec![0x7E],
            JmpCond::Greater => vec![0x7F],
            JmpCond::GreaterEqual => vec![0x7D],
        }
    }

    pub fn rel32(&self) -> Vec<u8> {
        match self {
            JmpCond::Equal => vec![0x0F, 0x84],
            JmpCond::NotEqual => vec![0x0F, 0x85],
            JmpCond::Less => vec![0x0F, 0x8C],
            JmpCond::LessEqual => vec![0x0F, 0x8E],
            JmpCond::Greater => vec![0x0F, 0x8F],
            JmpCond::GreaterEqual => vec![0x0F, 0x8D],
        }
    }

    pub const fn rel8_op_size(&self) -> usize {
        1
    }

    pub const fn rel32_op_size(&self) -> usize {
        2
    }
}

impl JmpInsn {
    pub fn opcode(&self) -> Vec<u8> {
        match self {
            Self::Short(_) => vec![0xEB],
            Self::Near(_) => vec![0xE9],
            Self::Indirect(_) => vec![0xFF], // FF /4

            Self::Cond8(c, _) => c.rel8(),
            Self::Cond32(c, _) => c.rel32(),
        }
    }
}

impl const InsnInfo for JmpInsn {
    fn predict_size(&self) -> usize {
        match self {
            Self::Short(_) => 2,
            Self::Near(_) => 5,
            Self::Indirect(it) => 2 + it.added_bytes(),

            Self::Cond8(c, _) => c.rel8_op_size() + 1,
            Self::Cond32(c, _) => c.rel32_op_size() + 4,
        }
    }
}

impl InsnEncode for JmpInsn {
    fn encode(self) -> Vec<u8> {
        match self {
            JmpInsn::Cond8(_, to) => {
                let mut buf = Vec::new();

                buf.extend(self.opcode());
                buf.push(to as u8);

                buf
            }

            JmpInsn::Cond32(_, to) => {
                let mut buf = Vec::new();

                buf.extend(self.opcode());
                buf.extend(to.to_le_bytes());

                buf
            }

            JmpInsn::Short(to) => {
                let mut buf = Vec::new();

                buf.extend(self.opcode());
                buf.push(to as u8);

                buf
            }

            JmpInsn::Near(to) => {
                let mut buf = Vec::new();

                buf.extend(self.opcode());
                buf.extend(to.to_le_bytes());

                buf
            }

            JmpInsn::Indirect(to) => {
                let mut buf = Vec::new();

                if to.bit_width() == 64 {
                    if let Some(_reg) = to.reg() {
                        // NOTE: Seems like jmp defaults to 64-bit on x64 mode, so it seems like this isn't needed.
                        // buf.push(crate::insn::encode_rex_for_reg(reg));
                    } else {
                        // NOTE: You can technically do this by generating both a mov and a jmp
                        // insn, but this could mess up user state, so probably not a good idea.
                        panic!("far indirect jumps using immediate operands are not supported!");
                    }
                } else {
                    panic!("jumps must be to 64-bit addresses!");
                }

                let modrm = if to.is_value() {
                    panic!("jmp does not support immediate operands!")
                } else {
                    ModRm {
                        mod_: modrm(None, Some(to)),
                        reg: 4,
                        rm: to.id_bits(),
                    }
                };

                buf.extend(self.opcode());
                buf.push(modrm.encode());
                buf.extend(to.extra_bytes());

                buf
            }
        }
    }
}
