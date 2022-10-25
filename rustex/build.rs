fn main() {
    //println!("cargo:rustc-env=RUSTFLAGS=\"-C link-args=-Wl,-zstack-size=16777216\"");
    //println!("cargo:rustc-link-args=-Wl,-zstack-size=16777216");
    //println!("cargo:rustc-link-search=native=/home/jazzpirate/work/Software/RusTeX/rustex/lib");
    //println!("cargo:rustc-link-lib=static=pdfium");

    //let pwd_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    //let path = Path::new(&*pwd_dir).join("lib/");
    //println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    //println!("cargo:rustc-link-lib=dylib=add");
    // println!("cargo:rustc-link-lib=static=add");
    // println!("cargo:rerun-if-changed=src/hello.c");
   /* for lib in &["X11","Xau","xcb","Xdmcp"] {
        println!("cargo:rustc-link-lib=static={}",lib);
    }*/
    /*println!("cargo:rustc-link-lib=dylib=X11");
    println!("cargo:rustc-link-lib=dylib=jpeg");*/
}