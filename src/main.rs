use std::fmt;
use std::error::Error;
use std::os::raw::c_char;
use std::fs::{File, create_dir};
use std::io::prelude::*;
use serde::Deserialize;
use tempfile::tempdir;
use scraper::{Html, Selector};
use chrono::{DateTime, Local, Duration};
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
  platform::windows::WindowExtWindows,
  dpi::PhysicalSize,
};

mod obs;
use obs::{Scene, Source, Data, Output};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventSummaries {
  event_summaries: Vec<EventSummary>,
  shows: Vec<Show>,
  digital_files: Vec<DigitalFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EventSummary {
  location: i64,
  channel: i64,
  show: i64,
  start: DateTime<Local>,
  end: DateTime<Local>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Show {
  id: i64,
  cg_title: String,
  event_date: DateTime<Local>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DigitalFile {
  show: i64,
  aspect_ratio: i64,
}

#[derive(Debug, Deserialize)]
struct Settings {
  input: String,
  output: String,
}

#[derive(Debug, Clone)]
pub struct EventSummaryMissing;

impl fmt::Display for EventSummaryMissing {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "the event summary for the channel is missing")
  }
}

impl Error for EventSummaryMissing {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
  }
}

#[derive(Debug, Clone)]
pub struct ShowMissing;

impl fmt::Display for ShowMissing {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "the show for the current event is missing")
  }
}

impl Error for ShowMissing {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
  }
}

#[derive(Debug, Clone)]
pub struct DigitalFileMissing;

impl fmt::Display for DigitalFileMissing {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "the digital file for the show is missing")
  }
}

impl Error for DigitalFileMissing {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
  }
}

fn display_text(string: &str) -> Result<(), Box<dyn Error>> {
  let scene = Scene::new("main scene")?;

  let settings = Data::new()?;
  settings.set_string("text", string)?;
  let source = Source::new("text_gdiplus", "error text", Some(&settings), None)?;

  let item = scene.add(&source)?;
  item.set_scale(1.0, 1.0);
  item.set_pos(0.0, 0.0);

  obs::set_output_source(0, &scene.get_source()?);

  Ok(())
}

fn show_loop(vi_source: Source) -> Result<(), Box<dyn Error>> {
  loop {
    let resp = reqwest::blocking::get("https://cablecast.bectv.org/CablecastAPI/v1/eventsummaries?future=true&include=show%2Cdigitalfile&limit_per_channel=1")?
      .json::<EventSummaries>()?;
    
    println!("{:?}", resp);

    let summary = resp.event_summaries.iter().find(|x| x.location == 22 && x.channel == 1).ok_or(EventSummaryMissing)?;
    let show = resp.shows.iter().find(|x| x.id == summary.show).ok_or(ShowMissing)?;
    let file = resp.digital_files.iter().find(|x| x.show == summary.show).ok_or(DigitalFileMissing)?;

    let is_4by3 = file.aspect_ratio == 1;

    println!("{:?}", summary);
    println!("{:?}", show);
    println!("{:?}", file);

    let time_to_show = Local::now().signed_duration_since::<Local>(summary.start + Duration::seconds(-20)).num_seconds();
    if time_to_show > -300 && time_to_show < -10 {
      display_text(&format!("Next show at {}", summary.start.to_rfc2822()))?;

      println!("sleeping until start");
      let duration = summary.start.signed_duration_since(Local::now()) + Duration::seconds(-20);
      std::thread::sleep(duration.to_std()?);
    } else if time_to_show < -10 {
      display_text(&format!("Next show at {}", summary.start.to_rfc2822()))?;

      println!("sleeping for 5 minutes");
      std::thread::sleep(Duration::minutes(5).to_std()?);
      continue;
    }

    println!("{:?}", Local::now().signed_duration_since(summary.start));
    println!("{:?}", Local::now().signed_duration_since(summary.end));

    let mut path = dirs::document_dir().unwrap();
    path.push("scissors-templates");
    if !path.exists() {
      create_dir(path.clone())?
    }

    if is_4by3 {
      path.push("default-4x3.html");
    } else {
      path.push("default-16x9.html");
    }

    let dir = tempdir()?;

    if path.exists() {
      let scene = Scene::new("main scene")?;

      let mut html = String::new();
      File::open(path)?.read_to_string(&mut html)?;
      html = html.replace("{{cg_title}}", &show.cg_title);
      html = html.replace("{{event_date}}", &show.event_date.format("%B %d, %Y").to_string());

      let mut f = File::create(dir.path().join("template.html"))?;
      f.write_all(html.as_bytes())?;

      let mut x = 0.0;
      let mut y = 0.0;
      let mut width = 0.0;
      let mut height = 0.0;

      let document = Html::parse_document(&html);
      let selector = Selector::parse("#VIDEO > rect").unwrap();
      let element = document.select(&selector).next();
      if let Some(element) = element {
        x = element.value().attr("x").unwrap().parse::<f32>().unwrap();
        y = element.value().attr("y").unwrap().parse::<f32>().unwrap();
        width = element.value().attr("width").unwrap().parse::<f32>().unwrap();
        height = element.value().attr("height").unwrap().parse::<f32>().unwrap();
      }

      let settings = Data::new()?;
      settings.set_bool("is_local_file", true)?;
      settings.set_string("local_file", dir.path().join("template.html").to_str().unwrap())?;
      settings.set_int("width", 1920)?;
      settings.set_int("height", 1080)?;
      let bg_source = Source::new("browser_source", "background", Some(&settings), None)?;

      let item = scene.add(&bg_source)?;
      item.set_scale(1.0, 1.0);
      item.set_pos(0.0, 0.0);

      let item = scene.add(&vi_source)?;
      item.set_scale(width / if is_4by3 { 1440.0 } else { 1920.0 }, height / 1080.0);
      item.set_pos(x, y);

      if is_4by3 {
        item.set_crop(240, 0, 240, 0);
      }

      obs::set_output_source(0, &scene.get_source()?);
    } else {
      display_text(&format!("Error: Could not find {}", path.to_str().unwrap()))?;
    }

    println!("sleeping until end");
    let duration = summary.end.signed_duration_since(Local::now()) + Duration::seconds(10);
    std::thread::sleep(duration.to_std()?);
  }
}

extern {
  fn scissors_run_qt();
}

fn main() -> Result<(), Box<dyn Error>> {
  {
    println!("obs version {}", obs::get_version_string()?);

    assert!(obs::startup("en-US", None, None)?);

    unsafe {
      scissors_run_qt();
    }

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

    let mut config: Option<Settings> = None;
    let mut path = dirs::document_dir().unwrap();
    path.push("scissors-config.json");
    if path.exists() {
      config = Some(serde_json::from_reader(File::open(path)?)?);
    }

    let vi_source = Source::new("decklink-input", "video", None, None);
    let vi_source = if let Ok(vi_source) = vi_source {
      let props = vi_source.properties()?;
      let prop = props.get("device_hash");
      if let Ok(prop) = prop {
        if prop.list_item_count() != 1 {
          for i in 0..prop.list_item_count() {
            println!("{}", prop.list_item_name(i)?);
            println!("{}", prop.list_item_string(i)?);
          }
    
          let dname = prop.list_item_name(1)?;
          let mut dstr = prop.list_item_string(1)?.to_string();
    
          println!("Using if config not set: {}", dname);
          println!("Using if config not set: {}", dstr);
    
          if let Some(config) = config.as_ref() {
            dstr = config.input.clone();
          }

          println!("Using: {}", dstr);

          let settings = Data::new()?;
          // settings.set_string("device_name", dname)?;
          settings.set_string("device_hash", &dstr)?;
          settings.set_string("mode_name", "Auto")?;
          settings.set_int("mode_id", -1)?;
          settings.set_int("audio_connection", 1)?;
          settings.set_int("video_connection", 1)?;
          
          vi_source.update(Some(&settings));
  
          vi_source
        } else {
          let settings = Data::new()?;
          settings.set_string("file", "../../../1080img.jpg")?;

          Source::new("image_source", "video", Some(&settings), None)?
        }        
      } else {
        let settings = Data::new()?;
        settings.set_string("file", "../../../1080img.jpg")?;

        Source::new("image_source", "video", Some(&settings), None)?
      }
    } else {
      let settings = Data::new()?;
      settings.set_string("file", "../../../1080img.jpg")?;

      Source::new("image_source", "video", Some(&settings), None)?
    };

    let output = Output::new("decklink_output", "decklink output", None, None);
    if let Ok(output) = output {
      let props = vi_source.properties()?;
      let prop = props.get("device_hash");
      if let Ok(prop) = prop {
        let prop_count = prop.list_item_count();
        for i in 0..prop_count {
          println!("{}", prop.list_item_name(i)?);
          println!("{}", prop.list_item_string(i)?);
        }

        let dname = prop.list_item_name(prop_count - 1)?;
        let mut dstr = prop.list_item_string(prop_count - 1)?.to_string();

        println!("Output using if config not set: {}", dname);
        println!("Output using if config not set: {}", dstr);

        if let Some(config) = config {
          dstr = config.output;
        }

        println!("Output using: {}", dstr);

        let settings = Data::new()?;
        // settings.set_string("device_name", dname)?;
        settings.set_string("device_hash", &dstr)?;
        settings.set_string("mode_name", "1080i59.94")?;
        settings.set_int("mode_id", 12)?;

        output.update(Some(&settings));

        assert!(output.start());
      }
    }

    std::thread::spawn(move || {
      show_loop(vi_source).unwrap();
    });

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
      .with_title("scissors")
      .with_inner_size(PhysicalSize::new(1600, 900))
      .build(&event_loop)?;

    let display = obs::Display::new(Box::into_raw(Box::new(obs::gs_init_data {
      window: obs::gs_window { hwnd: window.hwnd() },
      cx: 1920,
      cy: 1080,
      format: obs::gs_color_format_GS_BGRA,
      zsformat: obs::gs_zstencil_format_GS_ZS_NONE,
      adapter: 0,
      num_backbuffers: 0,
    })), 0xBABABA)?;

    display.add_draw_callback(&mut |_x, _y| {
      obs::render_main_texture();
    });

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
          if size.width as f32 / size.height as f32 > 16.0 / 9.0 {
            display.resize((1080.0 * (size.width as f32 / size.height as f32)) as u32, 1080);
          } else {
            display.resize(1920, (1920.0 * (size.height as f32 / size.width as f32)) as u32);
          }
        }
        _ => (),
      }
    });
  }

  // unsafe {
  //   obs::obs_shutdown();
  //   println!("remaining allocs {:?}", obs::bnum_allocs());
  // }

  // Ok(())
}
