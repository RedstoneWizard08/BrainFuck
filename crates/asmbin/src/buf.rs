use std::ops::{Index, IndexMut};

use crate::insn::{Insn, InsnEncode, InsnInfo};

#[derive(Debug, Clone)]
pub struct InsnBuf {
    pub(crate) buf: Vec<Insn>,
}

impl InsnBuf {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn calculate_length(&self) -> u64 {
        self.buf.iter().map(|it| it.predict_size()).sum::<usize>() as u64
    }

    pub fn encode(self) -> Vec<u8> {
        let mut buf = vec![0u8; self.calculate_length() as usize];
        let mut pos = 0;

        self.buf.into_iter().for_each(|it| {
            let len = it.predict_size();
            let enc = it.encode();

            assert_eq!(len, enc.len(), "len ({len}) != enc ({}): {it:?}", enc.len());

            buf[pos..pos + len].copy_from_slice(&enc);
            pos += len;
        });

        buf
    }
}

impl IntoIterator for InsnBuf {
    type Item = Insn;
    type IntoIter = <Vec<Insn> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.buf.into_iter()
    }
}

impl Index<usize> for InsnBuf {
    type Output = Insn;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
    }
}

impl IndexMut<usize> for InsnBuf {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buf[index]
    }
}
