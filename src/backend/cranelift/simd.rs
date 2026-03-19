use crate::backend::cranelift::CodeGenerator;
use cranelift::prelude::{InstBuilder, MemFlags, Type, types};
use cranelift_module::Module;

impl<'a, M: Module> CodeGenerator<'a, M> {
    fn simd_sections(&mut self, arr: &[i8]) -> Vec<Vec<i8>> {
        let mut sections = Vec::new();

        match arr.len() {
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

    fn simd_ty(&self, size: usize) -> Type {
        match size {
            16 => types::I8X16,
            32 => types::I8X32,
            64 => types::I8X64,

            o => panic!("Invalid size for SIMD constant: {o}"),
        }
    }

    pub(super) fn unsafe_simd_add_arr_move(&mut self, arr: &Vec<i8>, pos: i64) {
        let base_addr = self.b.use_var(self.tape_ptr);
        let parts = self.simd_sections(arr);
        let mut offset = 0;

        for part in parts {
            if part.len() == 1 {
                let mut addr = base_addr;

                if offset != 0 {
                    let add = self.b.ins().iconst(self.ptr, offset);

                    addr = self.b.ins().iadd(addr, add);
                }

                let cur = self.b.ins().load(self.byte, MemFlags::new(), addr, 0);
                let val = self.b.ins().iadd_imm(cur, part[0] as i64);

                self.b.ins().store(MemFlags::new(), val, addr, 0);

                offset += 1;
            } else {
                let size = part.len();
                let mut addr = base_addr;

                if offset != 0 {
                    let add = self.b.ins().iconst(self.ptr, offset);

                    addr = self.b.ins().iadd(addr, add);
                }

                let ty = self.simd_ty(size);

                let handle = self
                    .b
                    .func
                    .dfg
                    .constants
                    .insert(part.iter().map(|it| *it as u8).collect::<Vec<_>>().into());

                let add = self.b.ins().vconst(ty, handle);
                let cur = self.b.ins().load(ty, MemFlags::new(), addr, 0);
                let val = self.b.ins().iadd(cur, add);

                self.b.ins().store(MemFlags::new(), val, addr, 0);

                offset += size as i64;
            }
        }

        let pos = self.b.ins().iadd_imm(base_addr, pos);

        self.b.def_var(self.tape_ptr, pos);
    }
}
