extern crate drm;
extern crate gbm;
extern crate sys;

mod cube_smooth;
mod egl;
mod es_matrix;

use sys::Card;

use drm::control::{crtc, framebuffer, Device as ControlDevice, connector::Info as ConnectorInfo, Mode, ResourceInfo,
    encoder::Info as EncoderInfo, crtc::Info as CrtcInfo};
use gbm::{Device, Format, BufferObjectFlags};
use drm::control::crtc::{Events, Event};

use std::ffi::CStr;
use std::os::unix::io::AsRawFd;

fn main() {
    let card = Card::open_global();
    let gbm = Device::new(card).unwrap();

    run(&gbm);
}


fn run(gbm: &Device<Card>) {
    let (connector, mode, _encoder, crtc) = get_resources(&*gbm);
    let pixel_format = Format::XRGB8888;

    let gbm_surface = gbm.create_surface::<Card>(
        mode.size().0 as u32,
        mode.size().1 as u32,
        pixel_format,
        BufferObjectFlags::SCANOUT | BufferObjectFlags::RENDERING
    ).unwrap();

    let (egl_display, egl_surface, gl_modelviewmatrix, gl_modelviewprojectionmatrix, gl_normalmatrix) =
        cube_smooth::init(&gbm, &mode, 0, &gbm_surface, pixel_format);

    unsafe { sys::gles2::ClearColor(0., 0.5, 0.5, 1.0) };
    unsafe { sys::gles2::Clear(sys::gles2::COLOR_BUFFER_BIT) };

    let mut i = 0;

    let _swap_result = unsafe { sys::egl::SwapBuffers(egl_display, egl_surface) };

    let mut bo = unsafe { gbm_surface.lock_front_buffer() }.unwrap();
    let fb_info = framebuffer::create(gbm, &*bo).unwrap();

    let _ = crtc::set(gbm, crtc.handle(), fb_info.handle(), &[connector.handle()], (0, 0), Some(mode)).unwrap();
    use std::time::Instant;

    let aspect = mode.size().1 as f32 / mode.size().0 as f32;

    loop {
        cube_smooth::draw(i, aspect, gl_modelviewmatrix, gl_modelviewprojectionmatrix, gl_normalmatrix);
        i += 1;

        unsafe { sys::egl::SwapBuffers(egl_display, egl_surface) };
        let next_bo = unsafe { gbm_surface.lock_front_buffer() }.unwrap();
        let fb_info = framebuffer::create(gbm, &*next_bo).unwrap();

        // * Here you could also update drm plane layers if you want
        // * hw composition

        crtc::page_flip(
            &*gbm,
            crtc.handle(),
            fb_info.handle(),
            &[crtc::PageFlipFlags::PageFlipEvent],
        ).expect("Failed to queue Page Flip");

        let mut events: Events;
        let mut waiting_for_flip = true;
        while waiting_for_flip {
            events = crtc::receive_events(&*gbm).unwrap();
            for event in events {
                match event {
                    Event::Vblank(_s) => {}, //println!("VblankEvent:{}", s.frame),
                    Event::PageFlip(_s) => {
                        // println!("PageFlipEvent:{}", s.frame);
                        waiting_for_flip = false;
                    }
                    Event::Unknown(_s) => {}, //println!("unkonw event:{:?}", s),
                }
            }
        }

        bo = next_bo;
    }
}

fn get_resources(card: &Card) -> (ConnectorInfo, Mode, EncoderInfo, CrtcInfo) {
    let resources = card.resource_handles().unwrap();

    let connector = resources.connectors().iter().find_map(|&c| {
        if let Ok(c) = ConnectorInfo::load_from_device(card, c) {
            if c.connection_state() == drm::control::connector::State::Connected && c.size().0 > 0 && c.size().1 > 0 {
                return Some(c);
            }
        }

        None
    }).unwrap();

    let modes = connector.modes();
    let modes = &mut modes.to_owned();
    modes.sort_by(|a, b| {
        /*if a.is_preferred() != b.is_preferred() {
            a.is_preferred().cmp(&b.is_preferred()).reverse()
        } else*/ if a.size().0 as u32 * a.size().1 as u32 != b.size().0 as u32 * b.size().1 as u32 {
            (a.size().0 as u32 * a.size().1 as u32).cmp(&(b.size().0 as u32 * b.size().1 as u32)).reverse()
        } else {
            a.vrefresh().cmp(&b.vrefresh()).reverse()
        }
    });

    let mode = modes.iter().next().unwrap();

    println!("size {:?}, clock {:?}, hsync {:?}, vsync {:?}, hskew {:?}, vscan {:?}, vrefresh {:?}, pref {}, {}",
        mode.size(), mode.clock(), mode.hsync(), mode.vsync(), mode.hskew(),
        mode.vscan(), mode.vrefresh(), mode.is_preferred(), mode.name().to_string_lossy().into_owned());

    let encoder = EncoderInfo::load_from_device(card, connector.current_encoder().unwrap()).unwrap();

    let ctrc_handle = encoder.current_crtc().unwrap();
    let crtc = CrtcInfo::load_from_device(card, ctrc_handle).unwrap();

    (connector, *mode, encoder, crtc)
}
