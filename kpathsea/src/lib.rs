#![deny(missing_docs)]
//! High-level Rust API for working with the kpathsea file-searching library for TeX

use kpathsea_sys::*;
use std::ffi::{CStr,CString};
use which::which;
use lazy_static::lazy_static;

lazy_static! {
  /// loads the kpathsea library
  pub static ref kpathsealib : Kpathsea = {
    #[cfg(unix)]
    let path = "libkpathsea.so";
    #[cfg(windows)]
    let path = "kpathsealib.dll";
    unsafe { Kpathsea::new(/*process_path::get_executable_path().unwrap().to_str().unwrap().to_string() +*/path).unwrap() }

  };
}
/// External result type for handling library errors
pub type Result<T> = std::result::Result<T, &'static str>;

/// High-level interface struct for the kpathsea API
pub struct Kpaths(kpathsea);

// A kpathsea pointer is Send because it owns the data that it references. It
// is not Sync, because calling kpathsea functions on it is not thread-safe.
unsafe impl Send for Kpaths {}

/// Returns the path to the kpsewhich executable on the system.
fn get_kpsewhich_path() -> Result<CString> {
  let kpsewhich_path = which("kpsewhich")
    .map_err(|_| "Error finding kpsewhich executable")?;
  let kpsewhich_path_str = kpsewhich_path.to_string_lossy();
  Ok(CString::new(kpsewhich_path_str.into_owned().as_str()).unwrap())
}

impl Kpaths {
  /// Obtain a new kpathsea struct, with metadata for the current rust executable
  pub fn new() -> Result<Self> {
    let kpse = unsafe { kpathsealib.kpathsea_new() };

    // kpathsea says we should pass in the current executable name to
    // kpathsea_set_program_name, but there are cases where this causes
    // kpathsea to fail to find the available TeX distribution. Instead, we use
    // the location of the kpsewhich executable, which ensures that we find the
    // correct TeX distribution.
    let kpsewhich_path = get_kpsewhich_path()?;

    unsafe {
      kpathsealib.kpathsea_set_program_name(
        kpse,
        kpsewhich_path.as_ptr(),
        std::ptr::null()
      )
    }
    Ok(Kpaths(kpse))
  }

  /// For a given filename, try to guess the kpse format type from the file
  /// extension by looking it up in the format info table. This is a simplified
  /// version of the find_format function in kpsewhich.
  fn guess_format_from_filename(&self, filename: &str) -> kpse_file_format_type {
    // We go through each format type
    for format_type in 0..kpse_file_format_type_kpse_last_format {
      let format_info: &mut kpse_format_info_type =
        unsafe { &mut (*self.0).format_info[format_type as usize] };
      if format_info.type_.is_null() {
        // If this format hasn't been initialized yet, initialize it now.
        // Otherwise, it won't have the list of suffixes initialized.
        unsafe {
          kpathsealib.kpathsea_init_format(self.0, format_type as kpse_file_format_type);
        }
      }

      // First, we check the suffixes for each format type. The suffixes are
      // stored as an array of strings with a null pointer denoting the last
      // value. Also, the pointer to the array can itself be null if there are
      // no suffixes.
      let mut suffix_ptr = format_info.suffix;
      while !suffix_ptr.is_null() && !unsafe {*suffix_ptr}.is_null() {
        // Pull out the suffix
        let suffix_cstr = unsafe { CStr::from_ptr(*suffix_ptr) };
        let suffix = suffix_cstr.to_str().unwrap();

        // We check if the last suffix.len() characters of the filename are
        // equal to the suffix itself. If so, then we've found a type that
        // matches our filename!
        if filename.len() >= suffix.len() && filename.get(filename.len()-suffix.len()..) == Some(suffix) {
          return format_type as kpse_file_format_type;
        }

        // Go to the next suffix in the array.
        suffix_ptr = unsafe { suffix_ptr.offset(1) };
      }

      // Next, we check the alternate suffixes for each format type. This is
      // stored in the exact same way as the normal suffixes.
      // TODO(xymostech): factor this out into a function to avoid duplication
      let mut alt_suffix_ptr = format_info.alt_suffix;
      while !alt_suffix_ptr.is_null() && !unsafe {*alt_suffix_ptr}.is_null() {
        let alt_suffix_cstr = unsafe { CStr::from_ptr(*alt_suffix_ptr) };
        let alt_suffix = alt_suffix_cstr.to_str().unwrap();

        if filename.get(filename.len()-alt_suffix.len()..) == Some(alt_suffix) {
          return format_type as kpse_file_format_type;
        }

        alt_suffix_ptr = unsafe { alt_suffix_ptr.offset(1) };
      }
    }

    // If we don't find any matching suffixes, we guess that it's a tex file
    kpse_file_format_type_kpse_tex_format
  }

  /// Find a file base name, auto-completing with the standard TeX extensions if needed
  pub fn find_file(&self, name: &str) -> Option<String> {
    let c_name = CString::new(name).unwrap();

    let file_format_type = self.guess_format_from_filename(name);
    let c_filename_buf = unsafe { kpathsealib.kpathsea_find_file(
      self.0,
      c_name.as_ptr(),
      file_format_type,
      0
    )};
    
    if !c_filename_buf.is_null() {
      let c_filepath: &CStr = unsafe { CStr::from_ptr(c_filename_buf) };
      let filepath = c_filepath.to_str().unwrap().to_owned();
      if filepath.is_empty() { 
        None
      } else {
        Some(filepath)
      }
    } else {
      None
    }
  }
}

impl Drop for Kpaths {
  /// Cleanup the kpathsea pointer in the destructor
  fn drop(&mut self) {
    unsafe { kpathsealib.kpathsea_finish(self.0) };
  }
}
