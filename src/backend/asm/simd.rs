use itertools::Itertools;

use crate::backend::asm::{
    CodeGenerator, Rodata,
    insn::{AsmBuilder, Data, Reg, SimdReg},
};

impl<'a> CodeGenerator<'a> {
    fn simd_sections(&self, arr: &[i8]) -> Vec<Vec<i8>> {
        let mut sections = Vec::new();

        match arr.len() {
            // 1 | 128 | 256 | 512
            1 | 16 | 32 | 64 => sections.push(arr.to_vec()),

            2..16 => {
                sections.extend(arr.iter().map(|it| vec![*it]));
            }

            other => {
                let closest = if other > 64 {
                    64
                } else if other > 32 {
                    32
                } else if other > 16 {
                    16
                } else {
                    1
                };

                let part = arr[0..closest].to_vec();
                let rest = &arr[closest..];

                sections.push(part);
                sections.extend(self.simd_sections(rest));
            }
        }

        sections
    }

    pub(super) fn simd_add_arr_move(&mut self, arr: &Vec<i8>, pos: i64) {
        let sections = self.simd_sections(arr);
        let mut offset = 0;

        for part in sections {
            let size = part.len();

            if size == 1 {
            } else {
                let reg = match size {
                    16 => SimdReg::Xmm0,
                    32 => SimdReg::Ymm0,
                    64 => SimdReg::Zmm0,

                    _ => panic!("SIMD sectionizer fucked up :("),
                };

                let first = part[0];

                if part.iter().all(|it| *it == first) {
                    self.mov(Reg::Eax, first);
                    self.vpbroadcastb(reg, Reg::Eax);
                } else {
                    let data = Rodata {
                        name: format!("simd_op_{}", self.data),
                        align: size,
                        data: part
                            .into_iter()
                            .chunks(8)
                            .into_iter()
                            .map(|it| {
                                format!(
                                    ".byte {}",
                                    it.map(|it| format!("{it}")).collect::<Vec<_>>().join(", ")
                                )
                            })
                            .collect(),
                    };

                    self.vmovdqu8(reg, Data::RelLabel(data.name.clone()));
                    self.data += 1;
                    self.rodata.push(data);
                }

                self.vpaddb(reg, reg, self.ptr.ptr_offs(offset));
                self.vmovdqu8(self.ptr.ptr_offs(offset), reg);

                offset += size as i64;
            }
        }

        self.move_ptr(pos);
    }
}
