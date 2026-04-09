use crate::{backend::wasm::CodeGenerator, opt::action::OptAction};
use wasm_encoder::{BlockType, InstructionSink, MemArg};

impl<'a> CodeGenerator<'a> {
    pub(super) fn translate_loop<'i>(
        &mut self,
        b: &mut InstructionSink<'i>,
        actions: &Vec<OptAction>,
    ) {
        b.block(BlockType::Empty);
        b.loop_(BlockType::Empty);

        self.ptr(b)
            .i32_load8_u(MemArg {
                align: 0,
                memory_index: 0,
                offset: 0,
            })
            .i32_eqz()
            .br_if(1);

        for insn in actions {
            self.translate(b, insn);
        }

        b.br(0).end().end();
    }
}
