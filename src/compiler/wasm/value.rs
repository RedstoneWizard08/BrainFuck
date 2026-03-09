use crate::compiler::wasm::CodeGenerator;
use wasm_encoder::{InstructionSink, MemArg};

impl<'a> CodeGenerator<'a> {
    pub(super) fn add_slot<'i>(&self, b: &mut InstructionSink<'i>, amount: i64) {
        self.ptr(b);

        self.ptr(b)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .i32_const(amount as i32)
            .i32_add()
            .i32_store8(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            });
    }

    pub(super) fn add_slot_offset<'i>(
        &self,
        b: &mut InstructionSink<'i>,
        amount: i64,
        offset: i64,
    ) {
        self.ptr_offset(b, offset);

        self.ptr_offset(b, offset)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .i32_const(amount as i32)
            .i32_add()
            .i32_store8(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            });
    }

    pub(super) fn set_slot<'i>(&self, b: &mut InstructionSink<'i>, value: i64) {
        self.ptr(b).i32_const(value as i32).i32_store8(MemArg {
            align: 0,
            memory_index: 0,
            offset: 0,
        });
    }

    pub(super) fn set_slot_offset<'i>(&self, b: &mut InstructionSink<'i>, value: i64, offset: i64) {
        self.ptr_offset(b, offset)
            .i32_const(value as i32)
            .i32_store8(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            });
    }
}
