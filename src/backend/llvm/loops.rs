//! Loop construct code generation for the LLVM backend.

use crate::{backend::llvm::CodeGenerator, opt::action::OptAction};
use inkwell::IntPredicate;

impl<'a, 'c> CodeGenerator<'a, 'c> {
    pub(super) fn translate_loop(&mut self, actions: &Vec<OptAction>) {
        let check = self.cx.append_basic_block(self.func, "loop_check");
        let inner = self.cx.append_basic_block(self.func, "loop");
        let cont = self.cx.append_basic_block(self.func, "loop_after");

        self.b.build_unconditional_branch(check).unwrap();
        self.b.position_at_end(check);

        let zero = self.cx.i8_type().const_zero();
        let val = self.load_slot();

        let cond = self
            .b
            .build_int_compare(IntPredicate::NE, val, zero, "loop_eqz")
            .unwrap();

        self.b.build_conditional_branch(cond, inner, cont).unwrap();

        let check = self.b.get_insert_block().unwrap();

        self.b.position_at_end(inner);

        for insn in actions {
            self.translate(insn);
        }

        self.b.build_unconditional_branch(check).unwrap();
        self.b.position_at_end(cont);
    }

    pub(super) fn scan(&mut self, skip: i64) {
        // TODO: skip == 1 -> repne scasb
        let check = self.cx.append_basic_block(self.func, "scan_check");
        let inner = self.cx.append_basic_block(self.func, "scan");
        let cont = self.cx.append_basic_block(self.func, "scan_after");

        self.b.build_unconditional_branch(check).unwrap();
        self.b.position_at_end(check);

        let zero = self.cx.i8_type().const_zero();
        let val = self.load_slot();

        let cond = self
            .b
            .build_int_compare(IntPredicate::NE, val, zero, "scan_eqz")
            .unwrap();

        self.b.build_conditional_branch(cond, inner, cont).unwrap();

        let check = self.b.get_insert_block().unwrap();

        self.b.position_at_end(inner);

        self.move_ptr(skip);

        self.b.build_unconditional_branch(check).unwrap();
        self.b.position_at_end(cont);
    }
}
