#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use winapi::um::winuser::*;
use winapi::um::libloaderapi::*;
use std::ptr::{null, null_mut};
use std::mem::MaybeUninit;
use resvg::prelude::*;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// unsafe extern fn obs_enum_module_callback(param: *mut c_void, module: *mut obs_module_t) {
//   println!("enum! {}", *module);
// }

unsafe extern fn render_window(data: *mut c_void, cx: u32, cy: u32) {
  // obs_render_main_texture();
  obs_source_video_render(data as *mut obs_source_t);
}

fn win32_string( value : &str ) -> Vec<u16> {
  OsStr::new( value ).encode_wide().chain( once( 0 ) ).collect()
}

fn main() {
  unsafe {

  println!("obs version {}", unsafe { CStr::from_ptr(obs_get_version_string()).to_str().unwrap() });

  let startup = obs_startup("en-US\0".as_ptr() as *const c_char, null_mut(), null_mut());
  // println!("startup: {}", startup);
  assert!(startup);

  let reset_video =  unsafe {
    obs_reset_video(Box::into_raw(Box::new(obs_video_info {
      graphics_module: "libobs-d3d11\0".as_ptr() as *const c_char,

      fps_num: 30000,
      fps_den: 1001,

      base_width: 1920,
      base_height: 1080,

      output_width: 1920,
      output_height: 1080,


      // output_format: video_format_VIDEO_FORMAT_UYVY,
      output_format: video_format_VIDEO_FORMAT_NV12,

      adapter: 0,

      gpu_conversion: true,

      colorspace: video_colorspace_VIDEO_CS_DEFAULT,

      range: video_range_type_VIDEO_RANGE_DEFAULT,
      scale_type: obs_scale_type_OBS_SCALE_DISABLE,
    })))
  };

  assert!(reset_video == OBS_VIDEO_SUCCESS as i32);

  let reset_audio = unsafe {
    obs_reset_audio(Box::into_raw(Box::new(
      obs_audio_info {
        samples_per_sec: 48000,
        speakers: speaker_layout_SPEAKERS_STEREO,
      }
    )))
  };

  assert!(reset_audio);

  obs_load_all_modules();
  obs_post_load_modules();
  // obs_enum_modules(Some(obs_enum_module_callback), null_mut());

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

  let scene = obs_scene_create("main scene\0".as_ptr() as *const c_char);

  let settings = obs_data_create();
  obs_data_set_string(settings, "file\0".as_ptr() as *const c_char, "test.png\0".as_ptr() as *const c_char);

  let bg_source = obs_source_create("image_source\0".as_ptr() as *const c_char, "background\0".as_ptr() as *const c_char, settings, null_mut());

  let item = obs_scene_add(scene, bg_source);

  let mut scale = MaybeUninit::uninit();
  scissors_vec2_set(scale.as_mut_ptr(), 1.0, 1.0);
  obs_sceneitem_set_scale(item, scale.as_mut_ptr());

  let mut pos = MaybeUninit::uninit();
  scissors_vec2_set(pos.as_mut_ptr(), 0.0, 0.0);
  obs_sceneitem_set_pos(item, pos.as_mut_ptr());

  let settings = obs_data_create();
  // obs_data_set_string(settings, "device_name\0".as_ptr() as *const c_char, "DeckLink Quad (1)\0".as_ptr() as *const c_char);
  // obs_data_set_string(settings, "device_hash\0".as_ptr() as *const c_char, "3854888880_DeckLink Quad 2\0".as_ptr() as *const c_char);
  // obs_data_set_string(settings, "mode_name\0".as_ptr() as *const c_char, "Auto\0".as_ptr() as *const c_char);
  // obs_data_set_int(settings, "mode_id\0".as_ptr() as *const c_char, -1);
  // obs_data_set_int(settings, "audio_connection\0".as_ptr() as *const c_char, 1);
  // obs_data_set_int(settings, "video_connection\0".as_ptr() as *const c_char, 1);

  // obs_data_set_bool(settings, "looping\0".as_ptr() as *const c_char, true);

  // let vi_source = obs_source_create("ffmpeg_source\0".as_ptr() as *const c_char, "video\0".as_ptr() as *const c_char, settings, null_mut());
  let vi_source = obs_source_create("decklink-input\0".as_ptr() as *const c_char, "video\0".as_ptr() as *const c_char, null_mut(), null_mut());

  let props = obs_source_properties(vi_source);
  let prop = obs_properties_get(props, "device_hash\0".as_ptr() as *const c_char);
  println!("{}", obs_property_list_item_count(prop));

  for i in 0..obs_property_list_item_count(prop) {
    println!("{}", CStr::from_ptr(obs_property_list_item_name(prop, i)).to_str().unwrap());
    println!("{}", CStr::from_ptr(obs_property_list_item_string(prop, i)).to_str().unwrap());
  }

  let dname = obs_property_list_item_name(prop, 1);
  let dstr = obs_property_list_item_string(prop, 1);

  println!("Using: {}", CStr::from_ptr(dname).to_str().unwrap());
  println!("Using: {}", CStr::from_ptr(dstr).to_str().unwrap());

  let settings = obs_data_create();
  obs_data_set_string(settings, "device_name\0".as_ptr() as *const c_char, dname);
  obs_data_set_string(settings, "device_hash\0".as_ptr() as *const c_char, dstr);
  obs_data_set_string(settings, "mode_name\0".as_ptr() as *const c_char, "Auto\0".as_ptr() as *const c_char);
  obs_data_set_int(settings, "mode_id\0".as_ptr() as *const c_char, -1);
  obs_data_set_int(settings, "audio_connection\0".as_ptr() as *const c_char, 1);
  obs_data_set_int(settings, "video_connection\0".as_ptr() as *const c_char, 1);


  obs_source_update(vi_source, settings);

  // let prop = obs_properties_get(props, "mode_id\0".as_ptr() as *const c_char);
  // for i in 0..obs_property_list_item_count(prop) {
  //   println!("{}", CStr::from_ptr(obs_property_list_item_name(prop, i)).to_str().unwrap());
  //   println!("{}", CStr::from_ptr(obs_property_list_item_string(prop, i)).to_str().unwrap());
  // }

  // obs_data_set_string(settings, "device_hash\0".as_ptr() as *const c_char, obs_property_list_item_string(prop, 0));
  // obs_source_update(vi_source, settings);


  // panic!();


  let item = obs_scene_add(scene, vi_source);

  let mut scale = MaybeUninit::uninit();
  scissors_vec2_set(scale.as_mut_ptr(), width / 1440.0, height / 1080.0);
  obs_sceneitem_set_scale(item, scale.as_mut_ptr());

  let mut pos = MaybeUninit::uninit();
  scissors_vec2_set(pos.as_mut_ptr(), x, y);
  obs_sceneitem_set_pos(item, pos.as_mut_ptr());

  let settings = obs_data_create();
  obs_data_set_int(settings, "left\0".as_ptr() as *const c_char, 240);
  obs_data_set_int(settings, "right\0".as_ptr() as *const c_char, 240);
  // obs_data_set_int(settings, "width\0".as_ptr() as *const c_char, 1440);
  // obs_data_set_int(settings, "height\0".as_ptr() as *const c_char, 1080);

  let filter = obs_source_create("crop_filter\0".as_ptr() as *const c_char, "crop\0".as_ptr() as *const c_char, settings, null_mut());
  obs_source_filter_add(vi_source, filter);

  // obs_data_set_int(settings, "width\0".as_ptr() as *const c_char, width as i64);
  // obs_data_set_int(settings, "height\0".as_ptr() as *const c_char, height as i64);
  // obs_data_set_int(settings, "color\0".as_ptr() as *const c_char, 0xFF0000FF);
  // obs_data_set_string(settings, "color\0".as_ptr() as *const c_char, "#ffffff\0".as_ptr() as *const c_char);


  // let source = obs_source_create("color_source\0".as_ptr() as *const c_char, "some random source\0".as_ptr() as *const c_char, null_mut(), null_mut());


  // let item = null_mut();
  // let scale;

  // let mut scale = MaybeUninit::uninit();

  // scissors_vec2_set(scale.as_mut_ptr(), width / 1920.0, height / 1080.0);

  // let item = obs_scene_add(scene, source);
  // obs_sceneitem_set_scale(item, scale.as_mut_ptr());

  // let mut pos = MaybeUninit::uninit();

  // scissors_vec2_set(pos.as_mut_ptr(), x, y);
  // obs_sceneitem_set_pos(item, pos.as_mut_ptr());

  obs_set_output_source(0, obs_scene_get_source(scene));

  // obs_source_inc_showing(obs_scene_get_source(scene));

  let handle = unsafe {
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

    CreateWindowExW(
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
        null_mut() )
  };

  let display = unsafe {
    obs_display_create(Box::into_raw(Box::new(gs_init_data {
      window: gs_window { hwnd: handle as *mut std::ffi::c_void },
      cx: 1920,
      cy: 1080,
      format: gs_color_format_GS_BGRA,
      zsformat: gs_zstencil_format_GS_ZS_NONE,
      adapter: 0,
      num_backbuffers: 0,
    })), 0xFF0000)
  };

  unsafe {
    obs_display_add_draw_callback(display, Some(render_window), obs_scene_get_source(scene) as *mut c_void);

    // let vencoder = obs_video_encoder_create("obs_x264\0".as_ptr() as *const c_char, "test_x264\0".as_ptr() as *const c_char, null_mut(), null_mut());
	  // let aencoder = obs_audio_encoder_create("ffmpeg_aac\0".as_ptr() as *const c_char, "test_aac\0".as_ptr() as *const c_char, null_mut(), 0, null_mut());

    // obs_encoder_set_video(vencoder, obs_get_video());
    // obs_encoder_set_audio(aencoder, obs_get_audio());

    let output = obs_output_create("decklink_output\0".as_ptr() as *const c_char, "decklink output\0".as_ptr() as *const c_char, null_mut(), null_mut());

    let props = obs_source_properties(vi_source);
    let prop = obs_properties_get(props, "device_hash\0".as_ptr() as *const c_char);
    let prop_count = obs_property_list_item_count(prop);
    println!("{}", prop_count);

    for i in 0..prop_count {
      println!("{}", CStr::from_ptr(obs_property_list_item_name(prop, i)).to_str().unwrap());
      println!("{}", CStr::from_ptr(obs_property_list_item_string(prop, i)).to_str().unwrap());
    }

    let dname = obs_property_list_item_name(prop, prop_count - 1);
    let dstr = obs_property_list_item_string(prop, prop_count - 1);

    println!("Output using: {}", CStr::from_ptr(dname).to_str().unwrap());
    println!("Output using: {}", CStr::from_ptr(dstr).to_str().unwrap());

    let settings = obs_data_create();
    obs_data_set_string(settings, "device_name\0".as_ptr() as *const c_char, dname);
    obs_data_set_string(settings, "device_hash\0".as_ptr() as *const c_char, dstr);
    obs_data_set_string(settings, "mode_name\0".as_ptr() as *const c_char, "1080i59.94\0".as_ptr() as *const c_char);
    obs_data_set_int(settings, "mode_id\0".as_ptr() as *const c_char, 12);
    // obs_data_set_int(settings, "audio_connection\0".as_ptr() as *const c_char, 1);
    // obs_data_set_int(settings, "video_connection\0".as_ptr() as *const c_char, 1);

    // "mode_id": 12,
    // "mode_name": "1080i59.94",

    // let settings = obs_data_create();
    // obs_data_set_string(settings, "path\0".as_ptr() as *const c_char, "test.flv\0".as_ptr() as *const c_char);

    // obs_output_set_video_encoder(output, vencoder);
    // obs_output_set_audio_encoder(output, aencoder, 0);

    obs_output_update(output, settings);

    println!("{}", obs_output_start(output));


    loop {
      let mut message : MSG = std::mem::uninitialized();
      if GetMessageW( &mut message as *mut MSG, handle, 0, 0 ) > 0 {
          TranslateMessage( &message as *const MSG );
          DispatchMessageW( &message as *const MSG );
      } else {
          break;
      }
    }
  }

  }
}
