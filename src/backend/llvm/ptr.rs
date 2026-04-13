//! Pointer manipulation code generation for the LLVM backend.

use inkwell::values::BasicValue;

use crate::backend::llvm::CodeGenerator;

impl<'a, 'c> CodeGenerator<'a, 'c> {
    pub(super) fn set_move(&mut self, value: i64, offset: i64) {
        let ptr = self
            .b
            .build_load(self.ptr_ty, self.tape, "add_move_load_ptr")
            .unwrap()
            .into_pointer_value();

        let val = self.cx.i8_type().const_int(value as u64, true);
        let val = self.b.build_store(ptr, val).unwrap();

        val.set_metadata(self.tbaa_access, self.tbaa_kind).unwrap();

        let off = self.cx.i64_type().const_int(offset as u64, true);

        let new_ptr = unsafe {
            self.b
                .build_in_bounds_gep(self.cx.i8_type(), ptr, &[off], "add_move_gep")
                .unwrap()
        };

        self.b.build_store(self.tape, new_ptr).unwrap();
    }

    pub(super) fn add_move(&mut self, amount: i64, offset: i64) {
        let ptr = self
            .b
            .build_load(self.ptr_ty, self.tape, "add_move_load_ptr")
            .unwrap()
            .into_pointer_value();

        let val = self
            .b
            .build_load(self.cx.i8_type(), ptr, "add_move_load")
            .unwrap()
            .into_int_value();

        val.as_instruction_value()
            .unwrap()
            .set_metadata(self.tbaa_access, self.tbaa_kind)
            .unwrap();

        let add = self.cx.i8_type().const_int(amount as u64, true);
        let res = self.b.build_int_add(val, add, "add_move_add").unwrap();
        let val = self.b.build_store(ptr, res).unwrap();

        val.set_metadata(self.tbaa_access, self.tbaa_kind).unwrap();

        let off = self.cx.i64_type().const_int(offset as u64, true);

        let new_ptr = unsafe {
            self.b
                .build_in_bounds_gep(self.cx.i8_type(), ptr, &[off], "add_move_gep")
                .unwrap()
        };

        self.b.build_store(self.tape, new_ptr).unwrap();
    }

    pub(super) fn move_ptr(&mut self, amount: i64) {
        self.b
            .build_store(self.tape, self.get_offset_ptr(amount))
            .unwrap();
    }
}
