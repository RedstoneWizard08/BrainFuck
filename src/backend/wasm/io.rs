use crate::backend::wasm::CodeGenerator;
use wasm_encoder::{InstructionSink, MemArg};

impl<'a> CodeGenerator<'a> {
    pub(super) fn print_slot<'i>(&self, b: &mut InstructionSink<'i>) {
        self.ptr(b)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .call(self.putchar);
    }

    pub(super) fn print_slot_offset<'i>(&self, b: &mut InstructionSink<'i>, offset: i64) {
        self.ptr_offset(b, offset)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .call(self.putchar);
    }

    pub(super) fn bulk_print<'i>(&self, b: &mut InstructionSink<'i>, n: i64) {
        // WASM doesn't support reusing arguments, to my knowledge
        for _ in 0..n {
            self.ptr(b)
                .i32_load8_u(MemArg {
                    align: 0,
                    memory_index: 0,
                    offset: 0,
                })
                .call(self.putchar);
        }
    }

    pub(super) fn bulk_print_offset<'i>(&self, b: &mut InstructionSink<'i>, n: i64, offset: i64) {
        // WASM doesn't support reusing arguments, to my knowledge
        for _ in 0..n {
            self.ptr_offset(b, offset)
                .i32_load8_u(MemArg {
                    align: 0,
                    memory_index: 0,
                    offset: 0,
                })
                .call(self.putchar);
        }
    }

    pub(super) fn input_slot<'i>(&self, b: &mut InstructionSink<'i>) {
        self.ptr(b).call(self.getchar).i32_store8(MemArg {
            align: 0,
            memory_index: 0,
            offset: 0,
        });
    }

    pub(super) fn input_slot_offset<'i>(&self, b: &mut InstructionSink<'i>, offset: i64) {
        self.ptr_offset(b, offset)
            .call(self.getchar)
            .i32_store8(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            });
    }
}
