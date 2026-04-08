use crate::reg::Reg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegDataRef {
    Direct(Reg),
    DirectValue(Reg),
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
            | RegDataRef::DirectValue(reg)
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
            | RegDataRef::DirectValue(reg)
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
            | RegDataRef::DirectValue(reg)
            | RegDataRef::RegOffset8(reg, _)
            | RegDataRef::RegOffset32(reg, _) => reg.id_bits(),

            _ => 0,
        }
    }

    pub const fn reg(&self) -> Option<Reg> {
        match self {
            RegDataRef::Direct(reg)
            | RegDataRef::DirectValue(reg)
            | RegDataRef::RegOffset8(reg, _)
            | RegDataRef::RegOffset32(reg, _) => Some(*reg),

            _ => None,
        }
    }

    pub const fn is_reg_read(&self) -> bool {
        match self {
            RegDataRef::DirectValue(_)
            | RegDataRef::RegOffset8(_, _)
            | RegDataRef::RegOffset32(_, _) => true,

            _ => false,
        }
    }

    pub const fn added_bytes(&self) -> usize {
        match self {
            RegDataRef::Direct(_) | RegDataRef::DirectValue(_) => 0,

            RegDataRef::RegOffset8(_, _) => 1,
            RegDataRef::RegOffset32(_, _) => 4,
            RegDataRef::Value8(_) => 1,
            RegDataRef::Value16(_) => 2,
            RegDataRef::Value32(_) => 4,
            RegDataRef::Value64(_) => 8,
        }
    }

    pub const fn needs_rex(&self) -> bool {
        self.needs_64() || self.bit_width() == 64
    }

    pub fn extra_bytes(&self) -> Vec<u8> {
        match self {
            Self::Direct(_) | Self::DirectValue(_) => vec![],
            Self::RegOffset8(_, o) => vec![*o],
            Self::RegOffset32(_, o) => o.to_le_bytes().to_vec(),
            Self::Value8(v) => vec![*v],
            Self::Value16(v) => v.to_le_bytes().to_vec(),
            Self::Value32(v) => v.to_le_bytes().to_vec(),
            Self::Value64(v) => v.to_le_bytes().to_vec(),
        }
    }
}
