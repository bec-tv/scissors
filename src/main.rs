
// #![allow(unused_variables)]

use std::os::raw::{c_char, c_void};
// use winapi::um::winuser::*;
// use winapi::um::libloaderapi::*;
use std::ptr::{null_mut};
use resvg::prelude::*;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
  platform::windows::WindowExtWindows,
  dpi::PhysicalSize,
};

mod obs;
use obs::{Scene, Source, Data, Output};


// unsafe extern fn render_window(data: *mut c_void, cx: u32, cy: u32) {
//   obs::render_main_texture();
// }

fn win32_string( value : &str ) -> Vec<u16> {
  OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}

fn maifn() -> Result<(), Box<dyn Error>> {
  {
    println!("obs version {}", obs::get_version_string()?);

    assert!(obs::startup("en-US", None, None)?);

    let reset_video = unsafe {
      obs::obs_reset_video(Box::into_raw(Box::new(obs::obs_video_info {
        graphics_module: "libobs-d3d11\0".as_ptr() as *const c_char,
        fps_num: 30000,
        fps_den: 1001,
        base_width: 1920,
        base_height: 1080,
        output_width: 1920,
        output_height: 1080,
        output_format: obs::video_format_VIDEO_FORMAT_NV12,
        adapter: 0,
        gpu_conversion: true,
        colorspace: obs::video_colorspace_VIDEO_CS_DEFAULT,
        range: obs::video_range_type_VIDEO_RANGE_DEFAULT,
        scale_type: obs::obs_scale_type_OBS_SCALE_BICUBIC,
      })))
    };

    assert!(reset_video == obs::OBS_VIDEO_SUCCESS as i32);

    assert!(unsafe {
      obs::obs_reset_audio(Box::into_raw(Box::new(
        obs::obs_audio_info {
          samples_per_sec: 48000,
          speakers: obs::speaker_layout_SPEAKERS_STEREO,
        }
      )))
    });

    obs::load_all_modules();
    obs::post_load_modules();

    let doc = roxmltree::Document::parse(include_str!("../test.svg")).unwrap();
    let elem = doc.descendants().find(|n| n.attribute("id") == Some("VIDEO")).unwrap();
    let rect = elem.descendants().find(|n| n.has_tag_name("rect")).unwrap();

    let x = rect.attribute("x").unwrap().to_string().parse::<f32>().unwrap();
    let y = rect.attribute("y").unwrap().to_string().parse::<f32>().unwrap();
    let width = rect.attribute("width").unwrap().to_string().parse::<f32>().unwrap();
    let height = rect.attribute("height").unwrap().to_string().parse::<f32>().unwrap();

    let mut opt = resvg::Options::default();
    opt.usvg.path = Some("../../../../test.svg".into());

    let rtree = usvg::Tree::from_file(&"../../../../test.svg", &opt.usvg).unwrap();
    let backend = resvg::default_backend();
    let mut img = backend.render_to_image(&rtree, &opt).unwrap();
    img.save_png(std::path::Path::new("test.png"));

    let scene = Scene::new("main scene")?;

    let settings = Data::new()?;
    settings.set_string("file", "test.png")?;

    let bg_source = Source::new("image_source", "background", Some(&settings), None)?;

    let is_4by3 = true;

    let item = scene.add(&bg_source)?;
    item.set_scale(1.0, 1.0);
    item.set_pos(0.0, 0.0);

    let vi_source = if false {
      let vi_source = Source::new("decklink-input", "video", None, None)?;

      let props = vi_source.properties()?;
      let prop = props.get("device_hash")?;

      for i in 0..prop.list_item_count() {
        println!("{}", prop.list_item_name(i)?);
        println!("{}", prop.list_item_string(i)?);
      }

      let dname = prop.list_item_name(1)?;
      let dstr = prop.list_item_string(1)?;

      println!("Using: {}", dname);
      println!("Using: {}", dstr);

      let settings = Data::new()?;
      settings.set_string("device_name", dname)?;
      settings.set_string("device_hash", dstr)?;
      settings.set_string("mode_name", "Auto")?;
      settings.set_int("mode_id", -1)?;
      settings.set_int("audio_connection", 1)?;
      settings.set_int("video_connection", 1)?;
      
      vi_source.update(Some(&settings));

      vi_source
    } else {
      let settings = Data::new()?;
      settings.set_string("file", "../../../../1080img.jpg")?;

      Source::new("image_source", "video", Some(&settings), None)?
    };

    let item = scene.add(&vi_source)?;
    item.set_scale(width / if is_4by3 { 1440.0 } else { 1920.0 }, height / 1080.0);
    item.set_pos(x, y);

    let filter;
    if is_4by3 {
      let settings = Data::new()?;
      settings.set_int("left", 240)?;
      settings.set_int("right", 240)?;

      filter = Some(Source::new("crop_filter", "crop", Some(&settings), None)?);
      vi_source.filter_add(&filter.as_ref().unwrap());
    } else {
      filter = None;
    }

    // let item = item.clone();
    // let filter = filter.map_or(None, |x| Some(x.clone()));
    // std::thread::spawn(move || {
    //   let mut x = x;
    //   let mut y = y;

    //   let mut width = width;
    //   let mut height = height;

    //   let mut crop: f64 = 480.0;

    //   loop {
    //     if crop > 0.0 {
    //       crop -= 1.6;
    //       width += 3.2;
    //     } else {
    //       crop = 0.0;
    //     }

    //     if x >= 0.0 {
    //       x -= 1.6;
    //     }

    //     if y >= 0.0 {
    //       y -= 0.9;
    //     }

    //     if width <= 1920.0 {
    //       width += 1.6;
    //     }

    //     if height <= 1080.0 {
    //       height += 0.9;
    //     }

    //     item.set_scale(width / if is_4by3 { 1920.0 - (2.0 * crop as f32) } else { 1920.0 }, height / 1080.0);
    //     item.set_pos(x, y);

    //     if let Some(filter) = &filter {
    //       let settings = Data::new().unwrap();
    //       settings.set_int("left", crop.ceil() as i64);
    //       settings.set_int("right", crop.ceil() as i64);

    //       filter.update(Some(&settings));
    //     }

    //     std::thread::sleep_ms(16);
    //   }
    // });

    obs::set_output_source(0, &scene.get_source()?);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title("scissors")
      .with_inner_size(PhysicalSize::new(1920, 1080))
      .build(&event_loop)?;

    let display = obs::Display::new(Box::into_raw(Box::new(obs::gs_init_data {
      window: obs::gs_window { hwnd: window.hwnd() /* handle as *mut std::ffi::c_void */ },
      cx: window.inner_size().width,
      cy: window.inner_size().height,
      format: obs::gs_color_format_GS_BGRA,
      zsformat: obs::gs_zstencil_format_GS_ZS_NONE,
      adapter: 0,
      num_backbuffers: 0,
    })), 0x000000)?;

    display.add_draw_callback(&mut |x, y| {
      obs::render_main_texture();
    });

    // let output = Output::new("decklink_output", "decklink output", None, None)?;

    // let props = vi_source.properties()?;
    // let prop = props.get("device_hash")?;
    // let prop_count = prop.list_item_count();
    // for i in 0..prop_count {
    //   println!("{}", prop.list_item_name(i)?);
    //   println!("{}", prop.list_item_string(i)?);
    // }

    // let dname = prop.list_item_name(prop_count - 1)?;
    // let dstr = prop.list_item_string(prop_count - 1)?;

    // println!("Output using: {}", dname);
    // println!("Output using: {}", dstr);

    // let settings = Data::new()?;
    // settings.set_string("device_name", dname);
    // settings.set_string("device_hash", dstr);
    // settings.set_string("mode_name", "1080i59.94");
    // settings.set_int("mode_id", 12);

    // output.update(Some(&settings));

    // assert!(output.start());

    event_loop.run(move |event, _, control_flow| {
      *control_flow = ControlFlow::Wait;

      match event {
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        
        Event::WindowEvent {
          event: WindowEvent::Resized(size),
          window_id,
        } if window_id == window.id() => {
          display.resize(size.width, size.height);
        }
        _ => (),
      }
    });
  }

  unsafe {
    obs::obs_shutdown();
    println!("remaining allocs {:?}", obs::bnum_allocs());
  }

  Ok(())
}

fn main() {
  maifn().unwrap();
}
