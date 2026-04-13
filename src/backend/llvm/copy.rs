//! Copy loop code generation for the LLVM backend.

use crate::backend::llvm::CodeGenerator;

impl<'a, 'c> CodeGenerator<'a, 'c> {
    pub(super) fn copy_loop(&mut self, values: &Vec<(i64, i64)>) {
        let val = self.load_slot();

        for (offset, mul) in values {
            let orig = self.load_slot_offs(*offset);

            let res = match *mul {
                1 => self.b.build_int_add(orig, val, "copy_loop_1_add").unwrap(),

                -1 => self
                    .b
                    .build_int_sub(orig, val, "copy_loop_neg_1_sub")
                    .unwrap(),

                other => {
                    let mul = self.cx.i8_type().const_int(other as u64, true);
                    let add = self.b.build_int_mul(val, mul, "copy_loop_mul").unwrap();

                    self.b.build_int_add(orig, add, "copy_loop_add").unwrap()
                }
            };

            self.set_slot_value_offs(*offset, res);
        }

        self.set_slot(0);
    }
}
