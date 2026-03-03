#[cfg(any(target_os = "linux", target_os = "android"))]
#[path = "linux.rs"]
pub mod platform;

#[cfg(windows)]
#[path = "win.rs"]
pub mod platform;

#[cfg(target_os = "android")]
#[path = "android.rs"]
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
