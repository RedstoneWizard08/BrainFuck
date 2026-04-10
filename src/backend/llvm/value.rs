use crate::backend::llvm::CodeGenerator;
use inkwell::values::{BasicValue, IntValue, PointerValue};

impl<'a, 'c> CodeGenerator<'a, 'c> {
    pub(super) fn get_offset_ptr(&mut self, offset: i64) -> PointerValue<'c> {
        let tape = self
            .b
            .build_load(self.ptr_ty, self.tape, "get_offset_ptr_load")
            .unwrap()
            .into_pointer_value();

        let off = self.cx.i64_type().const_int(offset as u64, true);

        unsafe {
            self.b
                .build_in_bounds_gep(self.cx.i8_type(), tape, &[off], "get_offset_ptr_gep")
                .unwrap()
        }
    }

    pub(super) fn load_slot(&mut self) -> IntValue<'c> {
        let ptr = self
            .b
            .build_load(self.ptr_ty, self.tape, "load_slot_ptr")
            .unwrap()
            .into_pointer_value();

        let val = self
            .b
            .build_load(self.cx.i8_type(), ptr, "load_slot")
            .unwrap()
            .into_int_value();

        val.as_instruction_value()
            .unwrap()
            .set_metadata(self.tbaa_access, self.tbaa_kind)
            .unwrap();

        val
    }

    pub(super) fn load_slot_offs(&mut self, offset: i64) -> IntValue<'c> {
        let val = self
            .b
            .build_load(
                self.cx.i8_type(),
                self.get_offset_ptr(offset),
                "load_slot_offs",
            )
            .unwrap()
            .into_int_value();

        val.as_instruction_value()
            .unwrap()
            .set_metadata(self.tbaa_access, self.tbaa_kind)
            .unwrap();

        val
    }

    pub(super) fn set_slot_value(&mut self, val: IntValue<'c>) {
        let ptr = self
            .b
            .build_load(self.ptr_ty, self.tape, "set_slot_value_ptr")
            .unwrap()
            .into_pointer_value();

        let val = self.b.build_store(ptr, val).unwrap();

        val.set_metadata(self.tbaa_access, self.tbaa_kind).unwrap();
    }

    pub(super) fn set_slot_value_offs(&mut self, offset: i64, val: IntValue<'c>) {
        let val = self
            .b
            .build_store(self.get_offset_ptr(offset), val)
            .unwrap();

        val.set_metadata(self.tbaa_access, self.tbaa_kind).unwrap();
    }

    pub(super) fn add_slot(&mut self, amount: i64) {
        let add = self.cx.i64_type().const_int(amount as u64, true);
        let add = add.const_truncate(self.cx.i8_type());
        let val = self.load_slot();
        let res = self.b.build_int_add(val, add, "add_slot").unwrap();

        self.set_slot_value(res);
    }

    pub(super) fn add_slot_offset(&mut self, amount: i64, offset: i64) {
        let add = self.cx.i64_type().const_int(amount as u64, true);
        let add = add.const_truncate(self.cx.i8_type());
        let val = self.load_slot_offs(offset);
        let res = self.b.build_int_add(val, add, "add_slot_offset").unwrap();

        self.set_slot_value_offs(offset, res);
    }

    pub(super) fn set_slot(&mut self, value: i64) {
        self.set_slot_value(self.cx.i8_type().const_int(value as u64, true));
    }

    pub(super) fn set_slot_offset(&mut self, value: i64, offset: i64) {
        self.set_slot_value_offs(offset, self.cx.i8_type().const_int(value as u64, true));
    }
}
