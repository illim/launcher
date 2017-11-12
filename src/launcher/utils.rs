

use std::env;
use std::fs::{self, File};
use std::process;
use std::path::Path;
use std::io::{self, Read, Write, ErrorKind};
use reqwest;
use crypto::digest::Digest;
use crypto::md5::Md5;
use errors::*;

pub fn get_home_dir() -> String {
  let home_dir = env::home_dir().unwrap_or_else(|| {
    error!("Impossible to get your home dir!");
    process::exit(1);
  });
  format!("{}",home_dir.display()).replace("\\", "\\\\")
}

pub fn download<F>(source : &str, path_str : &str, expected_md5_opt : Option<&str>, update_progress : F) -> Result<u64>
  where F : FnMut(u64) {
  let mut res : reqwest::Response = reqwest::get(source)?;
  if !res.status().is_success() {
    return Err(From::from("Failed to get index"));
  }
  let path         = Path::new(&path_str);
  let path_tmp_str = path_str.to_string() + ".tmp";
  let path_tmp     = Path::new(&path_tmp_str);
  let parent_path = path.parent().ok_or(format!("No parent found for {}", &path.to_string_lossy()))?;
  fs::create_dir_all(parent_path)?;
  let mut target    = File::create(path_tmp)?;
  let (length, md5) = copy(&mut res, &mut target, update_progress)?;
  match expected_md5_opt {
    Some(expected_md5) if md5 != expected_md5 =>
      Err(From::from(format!("Md5 mismatch: expecting {} got {}", expected_md5, md5))),
    _ => {
      fs::rename(path_tmp, path)?;
      Ok(length)
    }
  } 
}

fn copy<R: ?Sized, W: ?Sized, F>(reader: &mut R, writer: &mut W, mut update_progress : F) -> io::Result<(u64, String)>
  where R: Read, W: Write, F : FnMut(u64)
{
  let mut buf = [0; 2048];
  let mut written = 0;
  let mut md5 = Md5::new();
  loop {
    let len = match reader.read(&mut buf) {
      Ok(0) => return Ok((written, md5.result_str())),
      Ok(len) => len,
      Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
      Err(e) => return Err(e),
    };
    let b = &buf[..len];
    writer.write_all(b)?;
    md5.input(b);
    written += len as u64;
    update_progress(written);
  }
}
