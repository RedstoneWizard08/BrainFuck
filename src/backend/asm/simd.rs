use crate::backend::asm::CodeGenerator;

impl<'a> CodeGenerator<'a> {
    pub(super) fn unsafe_simd_add_arr_move(
        &mut self,
        _arr: &Vec<i8>,
        _pos: i64,
    ) {
        panic!("SIMD is currently unsupported using the ASM backend!");
    }
}
