use crate::insn::{InsnEncode, InsnInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyscallInsn;

impl const InsnInfo for SyscallInsn {
    fn predict_size(&self) -> usize {
        2
    }
}

impl InsnEncode for SyscallInsn {
    fn encode(self) -> Vec<u8> {
        vec![0x0F, 0x05]
    }
}
