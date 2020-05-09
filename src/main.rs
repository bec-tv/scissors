
// #![allow(unused_variables)]

use std::os::raw::{c_char, c_void};
use winapi::um::winuser::*;
use winapi::um::libloaderapi::*;
use std::ptr::{null_mut};
use resvg::prelude::*;
use std::error::Error;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

mod obs;
use obs::{Scene, Source, Data, Output};


unsafe extern fn render_window(data: *mut c_void, cx: u32, cy: u32) {
  // obs_source_video_render(data as *mut obs_source_t);
  obs::obs_render_main_texture();
}

fn win32_string( value : &str ) -> Vec<u16> {
  OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}

fn maifn() -> Result<(), Box<dyn Error>> {
  unsafe {
    println!("obs version {}", obs::get_version_string()?);

    assert!(obs::startup("en-US", None, None)?);

    let reset_video = obs::obs_reset_video(Box::into_raw(Box::new(obs::obs_video_info {
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
      scale_type: obs::obs_scale_type_OBS_SCALE_DISABLE,
    })));

    assert!(reset_video == obs::OBS_VIDEO_SUCCESS as i32);

    assert!(obs::obs_reset_audio(Box::into_raw(Box::new(
      obs::obs_audio_info {
        samples_per_sec: 48000,
        speakers: obs::speaker_layout_SPEAKERS_STEREO,
      }
    ))));

    obs::obs_load_all_modules();
    obs::obs_post_load_modules();

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

    // let scene = obs_scene_create("main scene\0".as_ptr() as *const c_char);
    let scene = Scene::new("main scene")?;

    let settings = Data::new()?;
    settings.set_string("file", "test.png")?;

    // let bg_source = obs_source_create("image_source\0".as_ptr() as *const c_char, "background\0".as_ptr() as *const c_char, settings, null_mut());
    let bg_source = Source::new("image_source", "background", Some(&settings), None)?;

    let is_4by3 = true;

    let item = scene.add(&bg_source)?;
    item.set_scale(1.0, 1.0);
    item.set_pos(0.0, 0.0);

    let vi_source = if false {
      let vi_source = Source::new("decklink-input", "video", None, None)?;

      // let props = obs_source_properties(vi_source);
      let props = vi_source.properties()?;
      // let prop = obs::obs_properties_get(props, "device_hash\0".as_ptr() as *const c_char);
      let prop = props.get("device_hash")?;
      // println!("{}",  prop.list_item_count());

      for i in 0..prop.list_item_count() {
        println!("{}", prop.list_item_name(i)?);
        println!("{}", prop.list_item_string(i)?);
      }

      let dname = prop.list_item_name(1)?;
      let dstr = prop.list_item_string(1)?;

      println!("Using: {}", dname);
      println!("Using: {}", dstr);

      // let settings = obs_data_create();
      // obs_data_set_string(settings, "device_name\0".as_ptr() as *const c_char, dname);
      // obs_data_set_string(settings, "device_hash\0".as_ptr() as *const c_char, dstr);
      // obs_data_set_string(settings, "mode_name\0".as_ptr() as *const c_char, "Auto\0".as_ptr() as *const c_char);
      // obs_data_set_int(settings, "mode_id\0".as_ptr() as *const c_char, -1);
      // obs_data_set_int(settings, "audio_connection\0".as_ptr() as *const c_char, 1);
      // obs_data_set_int(settings, "video_connection\0".as_ptr() as *const c_char, 1);

      let settings = Data::new()?;
      settings.set_string("device_name", dname)?;
      settings.set_string("device_hash", dstr)?;
      settings.set_string("mode_name", "Auto")?;
      settings.set_int("mode_id", -1)?;
      settings.set_int("audio_connection", 1)?;
      settings.set_int("video_connection", 1)?;
      
      // obs_source_update(vi_source, settings);
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

    if is_4by3 {
      let settings = Data::new()?;
      settings.set_int("left", 240)?;
      settings.set_int("right", 240)?;

      let filter = Source::new("crop_filter", "crop", Some(&settings), None)?;
      vi_source.filter_add(&filter);
    }

    obs::set_output_source(0, &scene.get_source()?);

    let name = win32_string( "name" );
    let title = win32_string( "title" );

    let hinstance = GetModuleHandleW( null_mut() );
    let wnd_class = WNDCLASSW {
      style : CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc : Some( DefWindowProcW ),
      hInstance : hinstance,
      lpszClassName : name.as_ptr(),
      cbClsExtra : 0,
      cbWndExtra : 0,
      hIcon: null_mut(),
      hCursor: null_mut(),
      hbrBackground: null_mut(),
      lpszMenuName: null_mut(),
    };

    RegisterClassW( &wnd_class );

    let handle = CreateWindowExW(
      0,
      name.as_ptr(),
      title.as_ptr(),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      1920,
      1080,
      null_mut(),
      null_mut(),
      hinstance,
      null_mut()
    );

    let display = obs::obs_display_create(Box::into_raw(Box::new(obs::gs_init_data {
      window: obs::gs_window { hwnd: handle as *mut std::ffi::c_void },
      cx: 1920,
      cy: 1080,
      format: obs::gs_color_format_GS_BGRA,
      zsformat: obs::gs_zstencil_format_GS_ZS_NONE,
      adapter: 0,
      num_backbuffers: 0,
    })), 0x000000);

    obs::obs_display_add_draw_callback(display, Some(render_window), null_mut());

    let output = Output::new("decklink_output", "decklink output", None, None)?;

    let props = vi_source.properties()?;
    let prop = props.get("device_hash")?;
    let prop_count = prop.list_item_count();
    for i in 0..prop_count {
      println!("{}", prop.list_item_name(i)?);
      println!("{}", prop.list_item_string(i)?);
    }

    let dname = prop.list_item_name(prop_count - 1)?;
    let dstr = prop.list_item_string(prop_count - 1)?;

    println!("Output using: {}", dname);
    println!("Output using: {}", dstr);

    let settings = Data::new()?;
    settings.set_string("device_name", dname);
    settings.set_string("device_hash", dstr);
    settings.set_string("mode_name", "1080i59.94");
    settings.set_int("mode_id", 12);

    output.update(Some(&settings));

    assert!(output.start());
    loop {
      let mut message : MSG = std::mem::uninitialized();
      if GetMessageW( &mut message as *mut MSG, handle, 0, 0 ) > 0 {
          TranslateMessage( &message as *const MSG );
          DispatchMessageW( &message as *const MSG );
      } else {
          break;
      }
    }

    Ok(())
  }
}

fn main() {
  maifn().unwrap();
}
