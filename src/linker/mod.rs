//! Platform-specific linker implementations for finalizing compiled binaries.
//!
//! This module provides linker wrappers for different operating systems,
//! allowing the compilation of object files into final executables.

#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "linux.rs"]
/// Linux/Android linker implementation
pub mod platform;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
/// macOS linker implementation
pub mod platform;

#[cfg(windows)]
#[path = "win.rs"]
/// Windows linker implementation
pub mod platform;

#[cfg(target_os = "android")]
#[path = "android.rs"]
/// Android-specific linker subplatform
pub mod subplatform;

#[cfg(not(target_os = "android"))]
#[path = "dummy.rs"]
pub mod subplatform;

pub use platform::*;

#[allow(unused_imports)]
pub use subplatform::*;

use which::which;

pub fn command_exists<T>(cmd: T) -> bool
where
    T: AsRef<str>,
{
    which(cmd.as_ref()).is_ok()
}
