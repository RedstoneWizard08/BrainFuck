use crate::{backend::cranelift::CodeGenerator, opt::action::OptAction};
use cranelift::prelude::{InstBuilder, IntCC};
use cranelift_module::Module;

impl<'a, M: Module> CodeGenerator<'a, M> {
    pub(super) fn translate_loop(&mut self, actions: &Vec<OptAction>) {
        let header = self.b.create_block();
        let body = self.b.create_block();
        let exit = self.b.create_block();

        self.b.ins().jump(header, &[]);
        self.b.switch_to_block(header);

        let value = self.read_from_arr();
        let cond = self.b.ins().icmp_imm(IntCC::NotEqual, value, 0);

        self.b.ins().brif(cond, body, &[], exit, &[]);

        self.b.switch_to_block(body);
        self.b.seal_block(body);

        for action in actions {
            self.translate(action);
        }

        self.b.ins().jump(header, &[]);
        self.b.switch_to_block(exit);
        self.b.seal_block(header);
        self.b.seal_block(exit);
    }
}
