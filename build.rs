use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
  Command::new("python")
    .arg("tools/build.py")
    .arg("build")
    .status()
    .expect("failed to execute build script");

  let qt_version = "5.10.1";
  let qt_path = format!("deps/qt/{}/msvc2017_64", qt_version);

  println!("cargo:rustc-link-search=obs-studio/build/libobs/RelWithDebInfo");
  println!("cargo:rustc-link-search=cbuild/RelWithDebInfo");
  println!("cargo:rustc-link-search={}/lib", qt_path);

  println!("cargo:rustc-link-lib=obs");
  println!("cargo:rustc-link-lib=static=cscissors");
  println!("cargo:rustc-link-lib=Qt5Widgets");
  println!("cargo:rustc-link-lib=Qt5Core");

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
