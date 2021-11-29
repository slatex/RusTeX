use kpathsea_sys::*;
use std::env::current_exe;
use std::ffi::{CStr,CString};

#[test]
fn find_latex() {
  let kpse = unsafe { kpathsea_new() };
  let current_exe_result = current_exe();
  assert!(current_exe_result.is_ok(), "We need the path to the current test executable.");
  let current_exe_path = current_exe_result.unwrap();
  let mut current_exe_str = current_exe_path.to_string_lossy();
  let program_name = CString::new(current_exe_str.to_mut().as_str()).unwrap();
  unsafe { kpathsea_set_program_name(kpse, program_name.as_ptr(), program_name.as_ptr()); }

  let c_filename_buf = unsafe { kpathsea_find_file(
    kpse,
    CString::new("article.cls").unwrap().as_ptr(),
    kpse_file_format_type_kpse_tex_format,
    0
  )};
  
  let filename = if !c_filename_buf.is_null() {
    let c_str: &CStr = unsafe { CStr::from_ptr(c_filename_buf) };
    c_str.to_str().unwrap().to_owned()
  } else {
    String::new()
  };
    
  if filename.ends_with("article.cls") {
    println!("Found {:?}", filename);
    assert!(true, format!("Found {:?}", filename));
  } else if filename.is_empty() {
    assert!(true, format!("Not found, is TeX/TeXlive installed and kpathsea properly setup?"));
    println!("Not found, is TeX/TeXlive installed and kpathsea properly setup?");
  } else {
    assert!(false, format!("Unexpected filename returned: {:?}", filename));
  }
  
}