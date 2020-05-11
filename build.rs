use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
  Command::new("python")
    .arg("tools/build.py")
    .arg("build")
    .status()
    .expect("failed to execute build script");

  // println!("cargo:rustc-link-search=obs-studio/build/libobs/RelWithDebInfo");
  // println!("cargo:rustc-link-search=cbuild/Debug");
  println!("cargo:rustc-link-search=C:/Qt/5.14.2/msvc2017_64/lib");

  println!("cargo:rustc-link-lib=obs-studio/build/libobs/RelWithDebInfo/obs");
  println!("cargo:rustc-link-lib=cbuild/RelWithDebInfo/cscissors");
  // println!("cargo:rustc-link-lib=C:/Qt/5.14.2/msvc2017_64/lib/Qt5Widgets");

  // println!("cargo:rustc-link-lib=obs");
  // println!("cargo:rustc-link-lib=static=scissors");
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
