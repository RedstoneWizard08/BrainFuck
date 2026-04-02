use crate::reg::Reg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegDataRef {
    Direct(Reg),
    RegOffset8(Reg, u8),
    RegOffset32(Reg, u32),
    Value8(u8),
    Value16(u16),
    Value32(u32),
    Value64(u64),
}

impl RegDataRef {
    pub fn simplify(&mut self) {
        match self {
            RegDataRef::RegOffset8(reg, 0) | RegDataRef::RegOffset32(reg, 0) => {
                *self = RegDataRef::Direct(*reg);
            }

            _ => {}
        }
    }

    pub const fn is_value(&self) -> bool {
        match self {
            RegDataRef::Value8(_)
            | RegDataRef::Value16(_)
            | RegDataRef::Value32(_)
            | RegDataRef::Value64(_) => true,
            _ => false,
        }
    }

    pub const fn needs_64(&self) -> bool {
        match self {
            RegDataRef::Direct(reg)
            | RegDataRef::RegOffset8(reg, _)
            | RegDataRef::RegOffset32(reg, _) => reg.needs_64(),
            RegDataRef::Value8(_)
            | RegDataRef::Value16(_)
            | RegDataRef::Value32(_)
            | RegDataRef::Value64(_) => false,
        }
    }

    pub const fn bit_width(&self) -> usize {
        match self {
            RegDataRef::Direct(reg)
            | RegDataRef::RegOffset8(reg, _)
            | RegDataRef::RegOffset32(reg, _) => reg.bit_width(),
            RegDataRef::Value8(_) => 8,
            RegDataRef::Value16(_) => 16,
            RegDataRef::Value32(_) => 32,
            RegDataRef::Value64(_) => 64,
        }
    }

    pub const fn id_bits(&self) -> u8 {
        match self {
            RegDataRef::Direct(reg)
            | RegDataRef::RegOffset8(reg, _)
            | RegDataRef::RegOffset32(reg, _) => reg.id_bits(),
            RegDataRef::Value8(_)
            | RegDataRef::Value16(_)
            | RegDataRef::Value32(_)
            | RegDataRef::Value64(_) => 0,
        }
    }

    pub const fn added_bytes(&self) -> usize {
        match self {
            RegDataRef::Direct(_)
            | RegDataRef::RegOffset8(_, _)
            | RegDataRef::RegOffset32(_, _) => 0,

            RegDataRef::Value8(_) => 1,
            RegDataRef::Value16(_) => 2,
            RegDataRef::Value32(_) => 4,
            RegDataRef::Value64(_) => 8,
        }
    }
}
