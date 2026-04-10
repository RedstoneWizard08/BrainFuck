use crate::backend::llvm::CodeGenerator;
use inkwell::values::BasicValue;

impl<'a, 'c> CodeGenerator<'a, 'c> {
    pub(super) fn print_slot(&mut self) {
        let val = self.load_slot();

        let val = self
            .b
            .build_int_z_extend(val, self.cx.i32_type(), "print_slot_cast_to_i32")
            .unwrap();

        if !self.opts.no_io {
            self.b
                .build_call(
                    self.putchar,
                    &[val.as_basic_value_enum().into()],
                    "print_slot",
                )
                .unwrap();
        }
    }

    pub(super) fn print_slot_offset(&mut self, offset: i64) {
        let val = self.load_slot_offs(offset);

        let val = self
            .b
            .build_int_z_extend(val, self.cx.i32_type(), "print_slot_offset_cast_to_i32")
            .unwrap();

        if !self.opts.no_io {
            self.b
                .build_call(
                    self.putchar,
                    &[val.as_basic_value_enum().into()],
                    "print_slot_offset",
                )
                .unwrap();
        }
    }

    pub(super) fn bulk_print(&mut self, n: i64) {
        let val = self.load_slot();

        let val = self
            .b
            .build_int_z_extend(val, self.cx.i32_type(), "bulk_print_cast_to_i32")
            .unwrap();

        if !self.opts.no_io {
            for _ in 0..n {
                self.b
                    .build_call(
                        self.putchar,
                        &[val.as_basic_value_enum().into()],
                        "bulk_print",
                    )
                    .unwrap();
            }
        }
    }

    pub(super) fn bulk_print_offset(&mut self, n: i64, offset: i64) {
        let val = self.load_slot_offs(offset);

        let val = self
            .b
            .build_int_z_extend(val, self.cx.i32_type(), "bulk_print_offset_cast_to_i32")
            .unwrap();

        if !self.opts.no_io {
            for _ in 0..n {
                self.b
                    .build_call(
                        self.putchar,
                        &[val.as_basic_value_enum().into()],
                        "bulk_print_offset",
                    )
                    .unwrap();
            }
        }
    }

    pub(super) fn input_slot(&mut self) {
        todo!("LLVM backend: stdin");
    }

    pub(super) fn input_slot_offset(&mut self, _offset: i64) {
        todo!("LLVM backend: stdin");
    }
}
