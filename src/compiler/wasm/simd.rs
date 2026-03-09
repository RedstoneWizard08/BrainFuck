use crate::compiler::wasm::CodeGenerator;
use wasm_encoder::{InstructionSink, MemArg};

impl<'a> CodeGenerator<'a> {
    fn simd_sections(&mut self, arr: &[i8]) -> Vec<Vec<i8>> {
        let mut sections = Vec::new();

        match arr.len() {
            1 | 16 => sections.push(arr.to_vec()),

            2..16 => {
                sections.extend(arr.iter().map(|it| vec![*it]));
            }

            other => {
                let closest = if other > 16 { 16 } else { 1 };

                let part = arr[0..closest].to_vec();
                let rest = &arr[closest..];

                sections.push(part);
                sections.extend(self.simd_sections(rest));
            }
        }

        sections
    }

    pub(super) fn unsafe_simd_add_arr_move<'i>(
        &mut self,
        b: &mut InstructionSink<'i>,
        arr: &Vec<i8>,
        pos: i64,
    ) {
        let parts = self.simd_sections(arr);
        let mut offset = 0;

        for part in parts {
            if part.len() == 1 {
                self.ptr_offset(b, offset);
                self.ptr_offset(b, offset);

                b.i32_load8_u(MemArg {
                    align: 0,
                    memory_index: 0,
                    offset: 0,
                })
                .i32_const(part[0] as i32)
                .i32_add()
                .i32_store8(MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                });

                offset += 1;
            } else {
                let size = part.len();

                self.ptr_offset(b, offset);
                self.ptr_offset(b, offset);

                match size {
                    16 => {
                        let mut val: i128 = 0;

                        for i in 0..16 {
                            let lane_val = (part[i] as u8 as i128) << (i * 8);

                            val |= lane_val;
                        }

                        b.v128_load(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        })
                        .v128_const(val)
                        .i8x16_add()
                        .v128_store(MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        });
                    }

                    o => panic!("Invalid size for SIMD constant: {o}"),
                }

                offset += size as i64;
            }
        }

        self.ptr_offset(b, pos).local_set(self.tape_ptr);
    }
}
