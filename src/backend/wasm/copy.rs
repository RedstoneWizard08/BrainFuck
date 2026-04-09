use crate::backend::wasm::{CodeGenerator, PAGE_SIZE};
use wasm_encoder::{InstructionSink, MemArg};

impl<'a> CodeGenerator<'a> {
    pub(super) fn copy_loop<'i>(&self, b: &mut InstructionSink<'i>, values: &Vec<(i64, i64)>) {
        for (offset, mul) in values {
            self.ptr_offset(b, *offset)
                .i32_const(PAGE_SIZE - 1)
                .i32_and();

            self.ptr_offset(b, *offset)
                .i32_const(PAGE_SIZE - 1)
                .i32_and()
                .i32_load8_u(MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                });

            self.ptr(b).i32_load8_u(MemArg {
                offset: 0,
                align: 0,
                memory_index: 0,
            });

            b.i32_const(*mul as i32);
            b.i32_mul();
            b.i32_add();

            b.i32_store8(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            });
        }

        self.ptr(b).i32_const(0).i32_store8(MemArg {
            align: 0,
            memory_index: 0,
            offset: 0,
        });
    }
}
