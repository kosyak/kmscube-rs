use gbm::{Device, Format};

use std::ffi::CString;

use sys::{gles2 as gl, Card};

use crate::egl;
use crate::es_matrix::ESMatrix;

pub fn init(
    gbm: &Device<Card>,
    mode: &drm::control::Mode,
    samples: u32,
    gbm_surface: &gbm::Surface<Card>,
    pixel_format: Format
) -> (sys::egl::types::EGLDisplay, sys::egl::types::EGLSurface, i32, i32, i32) {
    let (result_d, result_s) = egl::init(gbm, samples, gbm_surface, pixel_format);

    let vertex_shader_source = r#"
        uniform mat4 modelviewMatrix;
        uniform mat4 modelviewprojectionMatrix;
        uniform mat3 normalMatrix;

        attribute vec4 in_position;
        attribute vec3 in_normal;
        attribute vec4 in_color;

        vec4 lightSource = vec4(2.0, 2.0, 20.0, 0.0);

        varying vec4 vVaryingColor;

        void main()
        {
            gl_Position = modelviewprojectionMatrix * in_position;
            vec3 vEyeNormal = normalMatrix * in_normal;
            vec4 vPosition4 = modelviewMatrix * in_position;
            vec3 vPosition3 = vPosition4.xyz / vPosition4.w;
            vec3 vLightDir = normalize(lightSource.xyz - vPosition3);
            float diff = max(0.0, dot(vEyeNormal, vLightDir));
            vVaryingColor = vec4(diff * in_color.rgb, 1.0);
        }
    "#;

    let fragment_shader_source = r#"
        precision mediump float;

        varying vec4 vVaryingColor;

        void main()
        {
            gl_FragColor = vVaryingColor;
        }
    "#;

    let vertices: [f32; 12 * 6] = [
        -1.0, -1.0, 1.0,
        1.0, -1.0, 1.0,
        -1.0, 1.0, 1.0,
        1.0, 1.0, 1.0, // front

        1.0, -1.0, -1.0,
        -1.0, -1.0, -1.0,
        1.0, 1.0, -1.0,
        -1.0, 1.0, -1.0, // back

        1.0, -1.0, 1.0,
        1.0, -1.0, -1.0,
        1.0, 1.0, 1.0,
        1.0, 1.0, -1.0, // right

        -1.0, -1.0, -1.0,
        -1.0, -1.0, 1.0,
        -1.0, 1.0, -1.0,
        -1.0, 1.0, 1.0, // left

        -1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, -1.0,
        1.0, 1.0, -1.0, // top

        -1.0, -1.0, -1.0,
        1.0, -1.0, -1.0,
        -1.0, -1.0, 1.0,
        1.0, -1.0, 1.0, // bottom
    ];

    let colors: [f32; 12 * 6] = [
        // front
        0.0,  0.0,  1.0, // blue
        1.0,  0.0,  1.0, // magenta
        0.0,  1.0,  1.0, // cyan
        1.0,  1.0,  1.0, // white
        // back
        1.0,  0.0,  0.0, // red
        0.0,  0.0,  0.0, // black
        1.0,  1.0,  0.0, // yellow
        0.0,  1.0,  0.0, // green
        // right
        1.0,  0.0,  1.0, // magenta
        1.0,  0.0,  0.0, // red
        1.0,  1.0,  1.0, // white
        1.0,  1.0,  0.0, // yellow
        // left
        0.0,  0.0,  0.0, // black
        0.0,  0.0,  1.0, // blue
        0.0,  1.0,  0.0, // green
        0.0,  1.0,  1.0, // cyan
        // top
        0.0,  1.0,  1.0, // cyan
        1.0,  1.0,  1.0, // white
        0.0,  1.0,  0.0, // green
        1.0,  1.0,  0.0, // yellow
        // bottom
        0.0,  0.0,  0.0, // black
        1.0,  0.0,  0.0, // red
        0.0,  0.0,  1.0, // blue
        1.0,  0.0,  1.0  // magenta
    ];

    let normals: [f32; 12 * 6] = [
        // front
        0.0, 0.0, 1.0, // forward
        0.0, 0.0, 1.0, // forward
        0.0, 0.0, 1.0, // forward
        0.0, 0.0, 1.0, // forward
        // back
        0.0, 0.0, -1.0, // backward
        0.0, 0.0, -1.0, // backward
        0.0, 0.0, -1.0, // backward
        0.0, 0.0, -1.0, // backward
        // right
        1.0, 0.0, 0.0, // right
        1.0, 0.0, 0.0, // right
        1.0, 0.0, 0.0, // right
        1.0, 0.0, 0.0, // right
        // left
        -1.0, 0.0, 0.0, // left
        -1.0, 0.0, 0.0, // left
        -1.0, 0.0, 0.0, // left
        -1.0, 0.0, 0.0, // left
        // top
        0.0, 1.0, 0.0, // up
        0.0, 1.0, 0.0, // up
        0.0, 1.0, 0.0, // up
        0.0, 1.0, 0.0, // up
        // bottom
        0.0, -1.0, 0.0, // down
        0.0, -1.0, 0.0, // down
        0.0, -1.0, 0.0, // down
        0.0, -1.0, 0.0  // down
    ];


    let gl_program = egl::create_program(vertex_shader_source, fragment_shader_source);
    assert!(gl_program >= 0);
    let gl_program = gl_program as u32;

    unsafe{ gl::BindAttribLocation(gl_program, 0, CString::new("in_position").unwrap().as_ptr()) };
    unsafe{ gl::BindAttribLocation(gl_program, 1, CString::new("in_normal").unwrap().as_ptr()) };
    unsafe{ gl::BindAttribLocation(gl_program, 2, CString::new("in_color").unwrap().as_ptr()) };

    unsafe { gl::LinkProgram(gl_program) };

    assert_ne!(unsafe {
        let mut ret = 0;
        gl::GetProgramiv(gl_program, gl::LINK_STATUS, &mut ret);
        ret
    }, 0);
    // if ret == 0 {
    //     char *log;

    //     printf("program linking failed!:\n");
    //     glGetProgramiv(program, GL_INFO_LOG_LENGTH, &ret);

    //     if (ret > 1) {
    //         log = malloc(ret);
    //         glGetProgramInfoLog(program, ret, NULL, log);
    //         printf("%s", log);
    //         free(log);
    //     }

    //     return -1;
    // }

    unsafe { gl::UseProgram(gl_program) };

    let gl_modelviewmatrix = unsafe {
        gl::GetUniformLocation(gl_program, CString::new("modelviewMatrix").unwrap().as_ptr())
    };
    let gl_modelviewprojectionmatrix = unsafe {
        gl::GetUniformLocation(gl_program, CString::new("modelviewprojectionMatrix").unwrap().as_ptr())
    };
    let gl_normalmatrix = unsafe {
        gl::GetUniformLocation(gl_program, CString::new("normalMatrix").unwrap().as_ptr())
    };

    unsafe { gl::Viewport(0, 0, mode.size().0 as i32, mode.size().1 as i32) };
    unsafe { gl::Enable(gl::CULL_FACE) };

    let gl_positionsoffset = 0;
    let size_of = std::mem::size_of::<f32>();
    let gl_colorsoffset = vertices.len() * size_of;
    let gl_normalsoffset = (vertices.len() + colors.len()) * size_of;
    let mut gl_vbo = 0;

    unsafe {
        gl::GenBuffers(1, &mut gl_vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, gl_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * size_of +
                colors.len() * size_of +
                normals.len() * size_of) as isize,
            std::ptr::null(),
            gl::STATIC_DRAW
        );
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            gl_positionsoffset,
            (vertices.len() * size_of) as isize,
            vertices.as_ptr() as *const _
        );
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            gl_colorsoffset as isize,
            (colors.len() * size_of) as isize,
            colors.as_ptr() as *const _
        );
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            gl_normalsoffset as isize,
            (normals.len() * size_of) as isize,
            normals.as_ptr() as *const _
        );
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, gl_positionsoffset as *const _);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, 0, gl_normalsoffset as *const _);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(2, 3, gl::FLOAT, gl::FALSE, 0, gl_colorsoffset as *const _);
        gl::EnableVertexAttribArray(2);
    };

    (result_d, result_s, gl_modelviewmatrix, gl_modelviewprojectionmatrix, gl_normalmatrix)
}

pub fn draw(
    i: u32,
    aspect: f32,
    modelviewmatrix: i32,
    modelviewprojectionmatrix: i32,
    normalmatrix: i32
) {
    /* clear the color buffer */
    unsafe { gl::ClearColor(0.0, 0.5, 0.5, 1.0) };
    unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };

    let modelview = ESMatrix::identity()
        .translate(0.0, 0.0, -8.0)
        .rotate(45.0 + (0.25 * i as f32), 1.0, 0.0, 0.0)
        .rotate(45.0 - (0.5 * i as f32), 0.0, 1.0, 0.0)
        .rotate(10.0 + (0.15 * i as f32), 0.0, 0.0, 1.0);

    let projection = ESMatrix::identity()
        .frustum(-2.8, 2.8, -2.8 * aspect, 2.8 * aspect, 6.0, 10.0);

    let modelviewprojection = ESMatrix::multiply(modelview, projection);

    let normal = [
        modelview.m()[0][0],
        modelview.m()[0][1],
        modelview.m()[0][2],
        modelview.m()[1][0],
        modelview.m()[1][1],
        modelview.m()[1][2],
        modelview.m()[2][0],
        modelview.m()[2][1],
        modelview.m()[2][2],
    ];

    let a = modelview.l();
    let b = modelviewprojection.l();

    unsafe { gl::UniformMatrix4fv(modelviewmatrix, 1, gl::FALSE, a.as_ptr()) };
    unsafe { gl::UniformMatrix4fv(modelviewprojectionmatrix, 1, gl::FALSE, b.as_ptr()) };
    unsafe { gl::UniformMatrix3fv(normalmatrix, 1, gl::FALSE, normal.as_ptr()) };

    unsafe {
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 4, 4);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 8, 4);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 12, 4);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 16, 4);
        gl::DrawArrays(gl::TRIANGLE_STRIP, 20, 4);
    }
}
