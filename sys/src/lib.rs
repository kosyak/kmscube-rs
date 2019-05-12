extern crate drm;
extern crate gbm;

use drm::control::Device as ControlDevice;

use std::fs::{File, OpenOptions};
use std::os::unix::io::{AsRawFd, RawFd};

use std::os::raw::{c_void, c_char, c_int};

#[link(name = "GLESv2")]
#[link(name = "EGL")]

#[link(name = "dl")]
extern {
    pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

#[allow(non_camel_case_types)]
pub mod egl {
    #![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]
    include!(concat!(env!("OUT_DIR"), "/egl.bindings.rs"));

    use std::os::raw;
    pub type khronos_utime_nanoseconds_t = raw::c_int;
    pub type khronos_uint64_t = u64;
    pub type khronos_ssize_t = isize;
    pub type EGLNativeDisplayType = *const raw::c_void;
    pub type EGLNativePixmapType = *const raw::c_void;
    pub type EGLNativeWindowType = *const raw::c_void;
    pub type EGLint = raw::c_int;
    pub type NativeDisplayType = *const raw::c_void;
    pub type NativePixmapType = *const raw::c_void;
    pub type NativeWindowType = *const raw::c_void;
}

pub mod gles2 {
    #![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]
    include!(concat!(env!("OUT_DIR"), "/gles2.bindings.rs"));
}

#[derive(Debug)]
// This is our customized struct that implements the traits in drm.
pub struct Card(File);

// Need to implement AsRawFd before we can implement drm::Device
impl AsRawFd for Card {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl drm::Device for Card {}
impl ControlDevice for Card {}

impl Card {
    pub fn open(path: &str) -> Self {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open(path).unwrap())
    }

    pub fn open_global() -> Self {
        Self::open("/dev/dri/card0")
    }

    // fn open_control() -> Self {
    //     Self::open("/dev/dri/controlD64")
    // }
}
