use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
  Command::new("python")
    .arg("tools/build.py")
    .arg("build")
    .status()
    .expect("failed to execute build script");

  println!("cargo:rustc-link-lib=obs-studio/build/libobs/Debug/obs");
  println!("cargo:rustc-link-lib=static=cbuild/Debug/scissors");

  println!("cargo:rerun-if-changed=wrapper.h");

  let bindings = bindgen::Builder::default()
    .header("wrapper.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}
