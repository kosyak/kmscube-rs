extern crate gl_generator;

use std::fs::File;
use std::path::Path;
use std::env;

use gl_generator::{Registry, Api, Profile, Fallbacks, StaticGenerator};

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut egl_file = File::create(&Path::new(&dest).join("egl.bindings.rs")).unwrap();
    let mut gles2_file = File::create(&Path::new(&dest).join("gles2.bindings.rs")).unwrap();

    Registry::new(Api::Egl, (1, 4), Profile::Core, Fallbacks::All, [
        "EGL_EXT_platform_base",
        "EGL_KHR_platform_gbm",
        "EGL_KHR_image_base",
        "EGL_KHR_fence_sync",
    ])
        .write_bindings(StaticGenerator, &mut egl_file)
        .unwrap();
    Registry::new(Api::Gles2, (2, 0), Profile::Core, Fallbacks::All, [
        "GL_OES_EGL_image",
    ])
        .write_bindings(StaticGenerator, &mut gles2_file)
        .unwrap();
}
