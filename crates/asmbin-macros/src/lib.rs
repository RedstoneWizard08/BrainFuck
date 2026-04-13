//! Procedural macros for the asmbin crate.
//!
//! This crate provides compile-time macro utilities for assembly binary generation.
//!
//! # Examples
//!
//! The `registers!` macro defines x86-64 registers with flexible syntax:
//!
//! ```ignore
//! registers! {
//!     0 = a & [seg16 = es; st];
//!     1 = c & [seg16 = cs; st];
//!     4 = {
//!         gp8 = rex ? spl : ah;
//!         gp.. = [sp, esp, rsp];
//!         st = true;
//!         seg16 = fs;
//!     };
//! }
//! ```

/// A procedural macro for defining x86-64 registers with flexible syntax.
///
/// This macro allows declarative specification of CPU registers with grouping,
/// naming conventions, and various flags.
///
/// # Examples
///
/// Define basic registers:
///
/// ```ignore
/// registers! {
///     0 = a & [seg16 = es; st];
///     1 = c;
///     2 = d;
/// }
/// ```
///
/// # Panics
///
/// Panics if the macro input cannot be parsed by the implementation.
#[proc_macro]
pub fn registers(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    asmbin_macros_impl::registers(input.into()).unwrap().into()
}
