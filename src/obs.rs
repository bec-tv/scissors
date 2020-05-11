#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::fmt;
use std::error::Error;
use std::ptr::{null, null_mut};
use std::mem::MaybeUninit;
use std::ffi::{CStr, CString, c_void};

#[derive(Debug, Clone)]
pub struct NullError;

impl fmt::Display for NullError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "obs api returned nullptr")
  }
}

impl Error for NullError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    None
  }
}

pub struct Scene {
  ptr: *mut obs_scene_t,
}

impl Scene {
  pub fn new(name: &str) -> Result<Self, Box<dyn Error>> {
    let name = CString::new(name)?;
    let ptr = unsafe {
      obs_scene_create(name.as_ptr())
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      unsafe { obs_scene_addref(ptr) }
      Ok(Self { ptr })
    }
  }

  pub fn add(&self, source: &Source) -> Result<SceneItem, Box<dyn Error>> {
    let ptr = unsafe {
      obs_scene_add(self.ptr, source.ptr)
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      Ok(SceneItem::from(ptr))
    }
  }

  pub fn get_source(&self) -> Result<Source, Box<dyn Error>> {
    let ptr = unsafe {
      obs_scene_get_source(self.ptr)
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      Ok(Source::from(ptr))
    }
  }
}

impl From<*mut obs_scene_t> for Scene {
  fn from(ptr: *mut obs_scene_t) -> Self {
    unsafe { obs_scene_addref(ptr) }
    Self { ptr }
  }
}

impl Clone for Scene {
  fn clone(&self) -> Self {
    Self::from(self.ptr)
  }
}

impl Drop for Scene {
  fn drop(&mut self) {
    unsafe {
      obs_scene_release(self.ptr);
    }
  }
}

unsafe impl Send for Scene {}

pub struct SceneItem {
  ptr: *mut obs_sceneitem_t,
}

impl SceneItem {
  pub fn set_scale(&self, x: f32, y: f32) {
    let mut vec = MaybeUninit::uninit();
    unsafe {
      scissors_vec2_set(vec.as_mut_ptr(), x, y);
      obs_sceneitem_set_scale(self.ptr, vec.as_mut_ptr());
    }
  }

  pub fn set_pos(&self, x: f32, y: f32) {
    let mut vec = MaybeUninit::uninit();
    unsafe {
      scissors_vec2_set(vec.as_mut_ptr(), x, y);
      obs_sceneitem_set_pos(self.ptr, vec.as_mut_ptr());
    }
  }

  pub fn set_visible(&self, visible: bool) {
    unsafe {
      obs_sceneitem_set_visible(self.ptr, visible);
    }
  }

  pub fn set_crop(&self, left: i32, top: i32, right: i32, bottom: i32) {
    unsafe {
      obs_sceneitem_set_crop(self.ptr, Box::into_raw(Box::new(obs_sceneitem_crop {
        left,
        top,
        right,
        bottom,
      })));
    }
  }
}

impl From<*mut obs_sceneitem_t> for SceneItem {
  fn from(ptr: *mut obs_sceneitem_t) -> Self {
    unsafe { obs_sceneitem_addref(ptr) }
    Self { ptr }
  }
}

impl Clone for SceneItem {
  fn clone(&self) -> Self {
    Self::from(self.ptr)
  }
}

impl Drop for SceneItem {
  fn drop(&mut self) {
    unsafe {
      obs_sceneitem_release(self.ptr);
    }
  }
}

unsafe impl Send for SceneItem {}

pub struct Source {
  ptr: *mut obs_source_t,
}

impl Source {
  pub fn new(id: &str, name: &str, settings: Option<&Data>, hotkey_data: Option<&Data>) -> Result<Self, Box<dyn Error>> {
    let id = CString::new(id)?;
    let name = CString::new(name)?;
    let ptr = unsafe {
      obs_source_create(
        id.as_ptr(),
        name.as_ptr(),
        settings.map_or(null_mut(), |x| x.ptr),
        hotkey_data.map_or(null_mut(), |x| x.ptr)
      )
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      unsafe { obs_source_addref(ptr) }
      Ok(Self { ptr })
    }
  }

  pub fn update(&self, settings: Option<&Data>) {
    unsafe {
      obs_source_update(self.ptr, settings.map_or(null_mut(), |x| x.ptr));
    }
  }

  pub fn properties(&self) -> Result<Properties, Box<dyn Error>> {
    let ptr = unsafe {
      obs_source_properties(self.ptr)
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      Ok(Properties::from(ptr))
    }
  }

  pub fn filter_add(&self, filter: &Source) {
    unsafe {
      obs_source_filter_add(self.ptr, filter.ptr);
    }
  }
}

impl From<*mut obs_source_t> for Source {
  fn from(ptr: *mut obs_source_t) -> Self {
    unsafe { obs_source_addref(ptr) }
    Self { ptr }
  }
}

impl Clone for Source {
  fn clone(&self) -> Self {
    Self::from(self.ptr)
  }
}

impl Drop for Source {
  fn drop(&mut self) {
    unsafe {
      obs_source_release(self.ptr);
    }
  }
}

unsafe impl Send for Source {}

pub struct Data {
  ptr: *mut obs_data_t,
}

impl Data {
  pub fn new() -> Result<Self, Box<dyn Error>> {
    let ptr = unsafe {
      obs_data_create()
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      unsafe { obs_data_addref(ptr) }
      Ok(Self { ptr })
    }
  }

  pub fn set_string(&self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    let key = CString::new(key)?;
    let value = CString::new(value)?;
    unsafe {
      obs_data_set_string(self.ptr, key.as_ptr(), value.as_ptr());
    }
    Ok(())
  }

  pub fn set_int(&self, key: &str, value: i64) -> Result<(), Box<dyn Error>> {
    let key = CString::new(key)?;
    unsafe {
      obs_data_set_int(self.ptr, key.as_ptr(), value);
    }
    Ok(())
  }

  pub fn set_bool(&self, key: &str, value: bool) -> Result<(), Box<dyn Error>> {
    let key = CString::new(key)?;
    unsafe {
      obs_data_set_bool(self.ptr, key.as_ptr(), value);
    }
    Ok(())
  }
}

impl From<*mut obs_data_t> for Data {
  fn from(ptr: *mut obs_data_t) -> Self {
    unsafe { obs_data_addref(ptr) }
    Self { ptr }
  }
}

impl Clone for Data {
  fn clone(&self) -> Self {
    Self::from(self.ptr)
  }
}

impl Drop for Data {
  fn drop(&mut self) {
    unsafe {
      obs_data_release(self.ptr);
    }
  }
}

unsafe impl Send for Data {}

pub struct Properties {
  ptr: *mut obs_properties_t,
}

impl Properties {
  pub fn get(&self, key: &str) -> Result<Property, Box<dyn Error>> {
    let key = CString::new(key)?;
    let ptr = unsafe {
      obs_properties_get(self.ptr, key.as_ptr())
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      Ok(Property{ ptr, parent: self })
    }
  }
}

impl From<*mut obs_properties_t> for Properties {
  fn from(ptr: *mut obs_properties_t) -> Self {
    Self { ptr }
  }
}

impl Drop for Properties {
  fn drop(&mut self) {
    unsafe {
      obs_properties_destroy(self.ptr);
    }
  }
}

unsafe impl Send for Properties {}

pub struct Property<'a> {
  ptr: *mut obs_property_t,
  parent: &'a Properties,
}

impl<'a> Property<'a> {
  pub fn list_item_count(&self) -> u64 {
    unsafe {
      obs_property_list_item_count(self.ptr)
    }
  }

  pub fn list_item_name(&self, index: u64) -> Result<&str, Box<dyn Error>> {
    let ptr = unsafe {
      obs_property_list_item_name(self.ptr, index)
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      Ok(unsafe { CStr::from_ptr(ptr).to_str()? })
    }
  }

  pub fn list_item_string(&self, index: u64) -> Result<&str, Box<dyn Error>> {
    let ptr = unsafe {
      obs_property_list_item_string(self.ptr, index)
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      Ok(unsafe { CStr::from_ptr(ptr).to_str()? })
    }
  }
}

pub struct Output {
  ptr: *mut obs_output_t,
}

impl Output {
  pub fn new(id: &str, name: &str, settings: Option<&Data>, hotkey_data: Option<&Data>) -> Result<Self, Box<dyn Error>> {
    let id = CString::new(id)?;
    let name = CString::new(name)?;
    let ptr = unsafe {
      obs_output_create(
        id.as_ptr(),
        name.as_ptr(),
        settings.map_or(null_mut(), |x| x.ptr),
        hotkey_data.map_or(null_mut(), |x| x.ptr)
      )
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      unsafe { obs_output_addref(ptr) }
      Ok(Self { ptr })
    }
  }

  pub fn update(&self, settings: Option<&Data>) {
    unsafe {
      obs_output_update(self.ptr, settings.map_or(null_mut(), |x| x.ptr));
    }
  }
  
  pub fn start(&self) -> bool {
    unsafe {
      obs_output_start(self.ptr)
    }
  }
}

impl From<*mut obs_output_t> for Output {
  fn from(ptr: *mut obs_output_t) -> Self {
    unsafe { obs_output_addref(ptr) }
    Self { ptr }
  }
}

impl Clone for Output {
  fn clone(&self) -> Self {
    Self::from(self.ptr)
  }
}

impl Drop for Output {
  fn drop(&mut self) {
    unsafe {
      obs_output_release(self.ptr);
    }
  }
}

unsafe impl Send for Output {}

pub struct Display {
  ptr: *mut obs_display_t,
}

impl Display {
  pub fn new(graphics_data: *mut gs_init_data, color: u32) -> Result<Self, Box<dyn Error>> {
    let ptr = unsafe {
      obs_display_create(graphics_data, color)
    };

    if ptr == null_mut() {
      Err(Box::new(NullError))
    } else {
      Ok(Self { ptr })
    }
  }

  pub fn add_draw_callback<F>(&self, callback: &mut F)
  where
    F: FnMut(u32, u32) + Send
  {
    extern "C" fn draw_callback<F>(data: *mut c_void, x: u32, y: u32)
    where
      F: FnMut(u32, u32) + Send,
    {
      let closure: &mut F = unsafe { &mut *(data as *mut F) };
      (*closure)(x, y);
    }

    unsafe {
      obs_display_add_draw_callback(self.ptr, Some(draw_callback::<F>), callback as *mut F as *mut c_void);
    }
  }

  pub fn resize(&self, cx: u32, cy: u32) {
    unsafe {
      obs_display_resize(self.ptr, cx, cy);
    }
  }
}

impl From<*mut obs_display_t> for Display {
  fn from(ptr: *mut obs_display_t) -> Self {
    Self { ptr }
  }
}


impl Drop for Display {
  fn drop(&mut self) {
    unsafe {
      obs_display_destroy(self.ptr);
    }
  }
}

unsafe impl Send for Display {}

pub fn get_version_string() -> Result<&'static str, Box<dyn Error>> {
  unsafe {
    Ok(CStr::from_ptr(obs_get_version_string()).to_str()?)
  }
}

pub fn startup(locale: &str, module_config_path: Option<&str>, store: Option<*mut profiler_name_store_t>) -> Result<bool, Box<dyn Error>> {
  let locale = CString::new(locale)?;
  let mut _module_config_path_string = CString::new("")?;
  let mut module_config_path_ptr = null();
  if let Some(string) = module_config_path {
    _module_config_path_string = CString::new(string)?;
    module_config_path_ptr = _module_config_path_string.as_ptr();
  }
  let store = store.unwrap_or(null_mut());
  unsafe {
    Ok(obs_startup(locale.as_ptr(), module_config_path_ptr, store))
  }
}

pub fn set_output_source(index: u32, source: &Source) {
  unsafe {
    obs_set_output_source(index, source.ptr);
  }
}

pub fn load_all_modules() {
  unsafe {
    obs_load_all_modules();
  }
}

pub fn post_load_modules() {
  unsafe {
    obs_post_load_modules();
  }
}

pub fn render_main_texture() {
  unsafe {
    obs_render_main_texture();
  }
}
