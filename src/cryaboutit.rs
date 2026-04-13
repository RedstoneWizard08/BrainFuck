//! This module consists of entirely unused functions just because I felt
//! a little silly and wanted to write some cursed Rust code.
//!
//! I seriously doubt any of this even works, but that just makes it even funnier.
//! This was just created to make actual Rust programmers cry.
//!
//! NEVER USE THIS IN PROD.
//!
//! Enjoy.

#[allow(unused, unsafe_op_in_unsafe_fn)]
mod _do_not_use {
    /// Doesn't implement copy? Now it does.
    unsafe fn forced_copy<A>(a: &A) -> A {
        let size = core::mem::size_of_val(a);
        let mut new = core::mem::zeroed::<A>();

        core::ptr::copy(a as *const A, &mut new as *mut A, 1);

        new
    }

    /// The classic.
    unsafe fn transmute_lifetime<'a, 'b, T>(val: &'a T) -> &'b T {
        &*(val as *const T)
    }

    /// If you're reading this, you have no life too! :) :) :)
    unsafe fn nolife<'a, T>(val: &'a T) -> T {
        core::mem::transmute_copy(&*(val as *const T))
    }

    /// Break the boundaries of same-sized types!
    unsafe fn force_transmute<A, B>(a: A) -> B {
        core::ptr::read_unaligned(&a as *const A as *const B)
    }

    /// Returning a sized [u8] is so much funnier than returning a Vec, am I right?
    unsafe fn bytes<T>(val: T) -> [u8; <T as std::mem::SizedTypeProperties>::SIZE] {
        force_transmute(val)
    }

    /// Don't ask. I don't even know what kind of use case there could possibly be for this.
    unsafe fn replace_but_funny<A, B>(dest: &mut A, src: B) -> A {
        let result = core::intrinsics::read_via_copy(dest);
        core::intrinsics::write_via_move(dest as *mut A as *mut B, src);
        result
    }
}
