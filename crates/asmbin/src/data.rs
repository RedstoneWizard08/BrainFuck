//! Data references for assembly operands (registers, memory, immediate values).

use std::ops::{Add, Sub};

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
            RegDataRef::Direct(reg) => reg.needs_64(),

            RegDataRef::DirectValue(_)
            | RegDataRef::RegOffset8(_, _)
            | RegDataRef::RegOffset32(_, _) => false, // depends on the other operand, this will adjust

            RegDataRef::Value8(_)
            | RegDataRef::Value16(_)
            | RegDataRef::Value32(_)
            | RegDataRef::Value64(_) => false,
        }
    }

    pub const fn bit_width(&self) -> usize {
        match self {
            RegDataRef::Direct(reg) => reg.bit_width(),

            RegDataRef::DirectValue(_)
            | RegDataRef::RegOffset8(_, _)
            | RegDataRef::RegOffset32(_, _) => 8, // depends on the other operand, this will adjust

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

impl From<u8> for RegDataRef {
    fn from(value: u8) -> Self {
        RegDataRef::Value8(value)
    }
}

impl From<u16> for RegDataRef {
    fn from(value: u16) -> Self {
        RegDataRef::Value16(value)
    }
}

impl From<u32> for RegDataRef {
    fn from(value: u32) -> Self {
        RegDataRef::Value32(value)
    }
}

impl From<u64> for RegDataRef {
    fn from(value: u64) -> Self {
        RegDataRef::Value64(value)
    }
}

impl From<Reg> for RegDataRef {
    fn from(value: Reg) -> Self {
        RegDataRef::Direct(value)
    }
}

impl From<[Reg; 1]> for RegDataRef {
    fn from(value: [Reg; 1]) -> Self {
        RegDataRef::DirectValue(value[0])
    }
}

impl Add<u8> for Reg {
    type Output = RegDataRef;

    fn add(self, rhs: u8) -> Self::Output {
        RegDataRef::RegOffset8(self, rhs)
    }
}

impl Add<u32> for Reg {
    type Output = RegDataRef;

    fn add(self, rhs: u32) -> Self::Output {
        RegDataRef::RegOffset32(self, rhs)
    }
}

impl Add<i64> for Reg {
    type Output = RegDataRef;

    fn add(self, rhs: i64) -> Self::Output {
        // TODO: Does this work? It might interpret the bits correctly...? Idk.
        if rhs <= u8::MAX as i64 {
            RegDataRef::RegOffset8(self, rhs as u8)
        } else {
            RegDataRef::RegOffset32(self, rhs as u32)
        }
    }
}

impl Sub<i64> for Reg {
    type Output = RegDataRef;

    fn sub(self, rhs: i64) -> Self::Output {
        self + (-rhs)
    }
}
