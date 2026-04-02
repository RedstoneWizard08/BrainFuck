#[proc_macro]
pub fn registers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    asmbin_macros_impl::registers(input.into()).unwrap().into()
}
