use std::env;
use std::path::{Path};

fn main() {
    /*let pwd_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&*pwd_dir).join("lib/");
    println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib=kpathsea");*/ /*
  if find_library("kpathsea").is_ok() {
    return;
  } else {
    if cfg!(kpathsea_docs_rs) { }
    else {
      panic!("Could not find kpathsea using pkg-config")
    }
  } */
}
