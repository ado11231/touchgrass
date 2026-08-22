//! Find the process that owns the focused window.

/// Disable focus detection and match every running process instead.
pub const ALL_PROCESSES_VARIABLE: &str = "TOUCHGRASS_ALL_PROCESSES";

/// Return the focused app's process ID, or `None` when it cannot be read.
pub fn pid() -> Option<u32> {
    if std::env::var_os(ALL_PROCESSES_VARIABLE).is_some_and(|value| !value.is_empty()) {
        return None;
    }
    platform::pid()
}

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::c_void;

    type CFTypeRef = *const c_void;

    /// Only list windows that are visible on screen.
    const ON_SCREEN_ONLY: u32 = 1 << 0;
    /// Skip the desktop picture and icons.
    const EXCLUDE_DESKTOP: u32 = 1 << 4;
    /// Start from no particular window.
    const NULL_WINDOW: u32 = 0;
    /// `kCFNumberSInt32Type`.
    const SINT32: isize = 3;
    /// Normal app windows live on layer zero; menus and the Dock sit above.
    const NORMAL_LAYER: i32 = 0;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGWindowListCopyWindowInfo(option: u32, relative_to: u32) -> CFTypeRef;
        static kCGWindowLayer: CFTypeRef;
        static kCGWindowOwnerPID: CFTypeRef;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFArrayGetCount(array: CFTypeRef) -> isize;
        fn CFArrayGetValueAtIndex(array: CFTypeRef, index: isize) -> CFTypeRef;
        fn CFDictionaryGetValue(dictionary: CFTypeRef, key: CFTypeRef) -> CFTypeRef;
        fn CFNumberGetValue(number: CFTypeRef, kind: isize, value: *mut c_void) -> bool;
        fn CFRelease(value: CFTypeRef);
    }

    fn number(dictionary: CFTypeRef, key: CFTypeRef) -> Option<i32> {
        let mut value = 0_i32;
        // SAFETY: `dictionary` is a live window dictionary and `key` a Core
        // Graphics key; the value, if present, is a CFNumber owned by it.
        unsafe {
            let number = CFDictionaryGetValue(dictionary, key);
            (!number.is_null()
                && CFNumberGetValue(number, SINT32, (&raw mut value).cast::<c_void>()))
            .then_some(value)
        }
    }

    /// Windows come back front to back, so the first normal one is focused.
    pub fn pid() -> Option<u32> {
        // SAFETY: The window list is a CFArray of CFDictionaries that stays
        // alive until the matching CFRelease below.
        unsafe {
            let windows = CGWindowListCopyWindowInfo(ON_SCREEN_ONLY | EXCLUDE_DESKTOP, NULL_WINDOW);
            if windows.is_null() {
                return None;
            }
            let found = (0..CFArrayGetCount(windows))
                .map(|index| CFArrayGetValueAtIndex(windows, index))
                .find(|window| number(*window, kCGWindowLayer) == Some(NORMAL_LAYER))
                .and_then(|window| number(window, kCGWindowOwnerPID))
                .and_then(|pid| u32::try_from(pid).ok());
            CFRelease(windows);
            found
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use std::process::{Command, Stdio};

    /// Ask `xdotool` for the focused X11 window; Wayland has no shared API.
    pub fn pid() -> Option<u32> {
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return None;
        }
        let output = Command::new("xdotool")
            .args(["getactivewindow", "getwindowpid"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        std::str::from_utf8(&output.stdout)
            .ok()?
            .trim()
            .parse()
            .ok()
    }
}
