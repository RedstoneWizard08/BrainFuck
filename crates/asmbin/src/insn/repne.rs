use crate::insn::{InsnEncode, InsnInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Scan {
    /// scasb
    ScanStringByte,
}

impl Scan {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Scan::ScanStringByte => vec![0xAE],
        }
    }

    pub const fn predict_size(&self) -> usize {
        match self {
            Scan::ScanStringByte => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepneInsn(pub Scan);

impl const InsnInfo for RepneInsn {
    fn predict_size(&self) -> usize {
        self.0.predict_size() + 1
    }
}

impl InsnEncode for RepneInsn {
    fn encode(self) -> Vec<u8> {
        let mut buf = vec![0xF2];

        buf.extend(self.0.encode());

        buf
    }
}
