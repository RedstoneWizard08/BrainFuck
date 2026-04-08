use crate::{
    data::RegDataRef,
    insn::{InsnEncode, InsnInfo, encode_insn},
    reg::Reg,
};

/// Load the effective address from [`Self::0`] into [`Self::1`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LeaInsn(pub RegDataRef, pub Reg);

impl LeaInsn {
    pub const fn opcode(&self) -> u8 {
        0x8D
    }
}

impl const InsnInfo for LeaInsn {
    fn predict_size(&self) -> usize {
        self.0.added_bytes() + 3
    }
}

impl InsnEncode for LeaInsn {
    fn encode(self) -> Vec<u8> {
        encode_insn(self.opcode(), self.1, Some(self.0), false)
    }
}
