#![allow(static_mut_refs)]

use crate::backend::CustomIo;

static mut STDOUT: Vec<u8> = Vec::new();
static mut STDIN: Vec<u8> = Vec::new();

const EOF: i32 = -1;

pub unsafe fn clear_io() {
    unsafe { STDOUT.clear() };
    unsafe { STDIN.clear() };
}

pub unsafe extern "C" fn buf_putchar(c: i32) -> i32 {
    // convert to a u8 since ascii characters are u8 anyway
    unsafe {
        STDOUT.push(c as u8);
    };
    c
}

pub unsafe extern "C" fn buf_getchar() -> i32 {
    if unsafe { STDIN.is_empty() } {
        EOF
    } else {
        unsafe { STDIN.remove(0) as i32 }
    }
}

/// Clears the stdin buffer, and swaps the stdout buffer with a new one, returning the previous one.
pub unsafe fn swap() -> Vec<u8> {
    unsafe { STDIN.clear() };

    let mut new = Vec::new();

    std::mem::swap(&mut new, unsafe { &mut STDOUT });

    new
}

pub struct BufTestingIo;

impl BufTestingIo {
    pub fn new() -> Self {
        unsafe { clear_io() };
        Self
    }

    pub fn load_stdin(&self, input: Vec<u8>) {
        unsafe {
            STDIN.extend(input);
            STDIN.push(0);
        };
    }

    pub fn finish(self) -> Vec<u8> {
        unsafe { swap() }
    }
}

impl CustomIo for BufTestingIo {
    fn getchar(&self) -> *const u8 {
        buf_getchar as *const u8
    }

    fn putchar(&self) -> *const u8 {
        buf_putchar as *const u8
    }
}
