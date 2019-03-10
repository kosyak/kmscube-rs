use gbm::{Device, Format};

use sys::*;
use std::ffi::{CStr, CString};

pub fn query_string(display: egl::types::EGLDisplay, name: egl::types::EGLenum) -> String {
    unsafe {
        CStr::from_ptr(egl::QueryString(display, name as i32)).to_string_lossy().into_owned()
    }
}

pub fn get_string(name: gles2::types::GLenum) -> String {
    unsafe {
        CStr::from_ptr(gles2::GetString(name)).to_string_lossy().into_owned()
    }
}

pub fn create_program(vs_src: &str, fs_src: &str) -> i32 {
    let mut ret = 0;

    let vertex_shader = unsafe { gles2::CreateShader(gles2::VERTEX_SHADER) };

    unsafe { gles2::ShaderSource(vertex_shader, 1, &CString::new(vs_src.as_bytes()).unwrap().as_ptr(), std::ptr::null()) };
    unsafe { gles2::CompileShader(vertex_shader) };

    unsafe { gles2::GetShaderiv(vertex_shader, gles2::COMPILE_STATUS, &mut ret) };
    if ret == 0 {
        dbg!("vertex shader compilation failed!");
        unsafe { gles2::GetShaderiv(vertex_shader, gles2::INFO_LOG_LENGTH, &mut ret) };
        if ret > 1 {
            let mut log = vec![0_u8; ret as usize];
            unsafe{ gles2::GetShaderInfoLog(vertex_shader, ret, std::ptr::null_mut(), log.as_mut_ptr()) };
            unsafe{ dbg!(CStr::from_ptr(log.as_ptr())) };
        }

        return -1;
    }

    let fragment_shader = unsafe { gles2::CreateShader(gles2::FRAGMENT_SHADER) };

    unsafe { gles2::ShaderSource(fragment_shader, 1, &CString::new(fs_src.as_bytes()).unwrap().as_ptr(), std::ptr::null()) };
    unsafe { gles2::CompileShader(fragment_shader) };

    unsafe { gles2::GetShaderiv(fragment_shader, gles2::COMPILE_STATUS, &mut ret) };
    if ret == 0 {
        // char *log;

        dbg!("fragment shader compilation failed!");
        unsafe { gles2::GetShaderiv(fragment_shader, gles2::INFO_LOG_LENGTH, &mut ret) };

        if ret > 1 {
            let mut log = vec![0_u8; ret as usize];
            unsafe{ gles2::GetShaderInfoLog(fragment_shader, ret, std::ptr::null_mut(), log.as_mut_ptr()) };
            unsafe{ dbg!(CStr::from_ptr(log.as_ptr())) };
        }

        return -1;
    }

    let program = unsafe { gles2::CreateProgram() };

    unsafe { gles2::AttachShader(program, vertex_shader) };
    unsafe { gles2::AttachShader(program, fragment_shader) };

    program as i32
}

pub fn init(
    gbm: &Device<Card>,
    samples: u32,
    gbm_surface: &gbm::Surface<Card>,
    pixel_format: Format
) -> (egl::types::EGLDisplay, egl::types::EGLSurface) {
    use gbm::AsRaw;

    let mut major: egl::EGLint = -1;
    let mut minor: egl::EGLint = -1;

    let egl_exts_client = query_string(egl::NO_DISPLAY, egl::EXTENSIONS);
    let ext_platform_base_address = if egl_exts_client.contains("EGL_EXT_platform_base") { unsafe {
        Some(egl::GetProcAddress(CString::new("eglGetPlatformDisplayEXT").unwrap().as_ptr()))
    } } else { None };

    let display = if ext_platform_base_address.is_some() {
        unsafe { egl::GetPlatformDisplayEXT(egl::PLATFORM_GBM_KHR, gbm.as_raw_mut() as *mut _, std::ptr::null()) }
    } else {
        unsafe { egl::GetDisplay(gbm.as_raw_mut() as *const _) }
    };

    assert_eq!(unsafe { egl::Initialize(display, &mut major, &mut minor) }, 1);

    let egl_exts_dpy = query_string(display, egl::EXTENSIONS);

    let _modifiers_supported = egl_exts_dpy.contains("EGL_EXT_image_dma_buf_import_modifiers");

    println!("Using display {:?} with EGL version {:?}.{:?}", display, major, minor);

    println!("===================================");
    println!("EGL information:");
    println!("  version: \"{}\"", query_string(display, egl::VERSION));
    println!("  vendor: \"{}\"", query_string(display, egl::VENDOR));
    println!("  client extensions: \"{}\"", egl_exts_client);
    println!("  display extensions: \"{}\"", egl_exts_dpy);
    println!("===================================");

    assert_eq!(unsafe {egl::BindAPI(egl::OPENGL_ES_API)}, 1);

    let mut config_size = -1;
    let mut matched_config_size = -1;

    let config_attribs = vec![
        egl::SURFACE_TYPE as i32, egl::WINDOW_BIT as i32,
        egl::RED_SIZE as i32, 1,
        egl::GREEN_SIZE as i32, 1,
        egl::BLUE_SIZE as i32, 1,
        egl::ALPHA_SIZE as i32, 0,
        egl::RENDERABLE_TYPE as i32, egl::OPENGL_ES2_BIT as i32,
        egl::SAMPLES as i32, samples as i32,
        egl::NONE as i32
    ];

    let context_attribs = vec![
        egl::CONTEXT_CLIENT_VERSION as i32, 2,
        egl::NONE as i32
    ];

    assert_eq!(unsafe {
        egl::GetConfigs(display, std::ptr::null_mut(), 0, &mut config_size)
    }, 1);

    let mut configs = vec![std::ptr::null() as egl::types::EGLConfig; config_size as usize];
    dbg!(config_size);

    assert_eq!(unsafe {
        egl::ChooseConfig(
            display,
            config_attribs.as_ptr(),
            configs.as_mut_ptr(),
            config_size,
            &mut matched_config_size
        )
    }, 1);

    let egl_config = configs.iter().find(|&&c| unsafe {
        let mut value = -1;
        egl::GetConfigAttrib(display, c, egl::NATIVE_VISUAL_ID as i32, &mut value);
        value
    } == pixel_format.as_ffi() as i32).unwrap();

    let egl_context = unsafe {
        egl::CreateContext(display, *egl_config, egl::NO_CONTEXT, context_attribs.as_ptr())
    };
    assert!(egl_context != std::ptr::null());

    let egl_surface = unsafe {
        egl::CreateWindowSurface(display, *egl_config, gbm_surface.as_raw() as *const _, std::ptr::null())
    };

    assert!(egl_surface != egl::NO_SURFACE);

    unsafe { egl::MakeCurrent(display, egl_surface, egl_surface, egl_context) };

    println!("OpenGL ES 2.x information:");
    println!("  version: \"{}\"", get_string(gles2::VERSION));
    println!("  shading language version: \"{}\"", get_string(gles2::SHADING_LANGUAGE_VERSION));
    println!("  vendor: \"{}\"", get_string(gles2::VENDOR));
    println!("  renderer: \"{}\"", get_string(gles2::RENDERER));
    println!("  extensions: \"{}\"", get_string(gles2::EXTENSIONS));
    println!("===================================");

    (display, egl_surface)
}
