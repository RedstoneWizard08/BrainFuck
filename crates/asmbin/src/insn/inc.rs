use crate::{
    data::RegDataRef,
    insn::{InsnEncode, InsnInfo, ModRm, encode_rex, modrm},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IncInsn(pub RegDataRef);

impl IncInsn {
    pub const fn opcode(&self) -> u8 {
        if self.0.bit_width() == 8 { 0xFE } else { 0xFF }
    }
}

impl const InsnInfo for IncInsn {
    fn predict_size(&self) -> usize {
        if self.0.needs_64() || self.0.bit_width() == 64 {
            3 + self.0.added_bytes()
        } else {
            2 + self.0.added_bytes()
        }
    }
}

impl InsnEncode for IncInsn {
    fn encode(mut self) -> Vec<u8> {
        self.0.simplify();

        let needs_rex = self.0.needs_64() || self.0.bit_width() == 64;
        let mut buf = Vec::new();

        if needs_rex {
            buf.push(encode_rex(&None, &Some(self.0)));
        }

        buf.push(self.opcode());

        buf.push(
            ModRm {
                mod_: modrm(None, Some(self.0)),
                reg: 0,
                rm: self.0.id_bits(),
            }
            .encode(),
        );

        buf.extend(self.0.extra_bytes());
        buf
    }
}
