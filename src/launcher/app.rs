extern crate hyper;

use std::fs::{self,File};
use std::path::Path;
use std::io::{Read, Result, Write, Error, ErrorKind};
use std::process;
use std::str;
use hyper::Client;
use launcher::config::{self, CommandConfig, IndexConfig, FileConfig, Index};
use crypto::digest::Digest;
use crypto::md5::Md5;

pub struct IndexState {
  pub current : Option<Index>,
  pub index : Index
}

pub fn execute_and_die(command : &CommandConfig) -> Result<()> {
  try!(execute(command));
  process::exit(0);
}

pub fn replace_index(index_config : &IndexConfig) {
  let tmpfile  = index_config.tmpfile();
  let tmp_path = Path::new(&tmpfile);

  if tmp_path.exists() {
    fs::rename(tmp_path, Path::new(&index_config.file)).expect("Failed rename index");
  }
}

pub fn get_index_state(index_config : &IndexConfig) -> Result<IndexState> {
  let tmpfile= index_config.tmpfile();

  try!(download(&index_config.source, &tmpfile, None, |_| {} ));
  Ok(
    IndexState {
      current : try!(config::load_index(index_config)),
      index   : try!(config::read_index(&index_config.tmpfile()))
    })
}

impl IndexState {

  pub fn has_diffs(&self) -> bool {
    match self.current {
      None => true,
      Some(ref current) => {
        self.index.files
          .iter()
          .any(|file : &FileConfig| exists_diff(file, current))
      }
    }
  }

}

pub fn filter_diffs(reffiles : Vec<FileConfig>, current : &Index) -> Vec<FileConfig> {
  reffiles
    .into_iter()
    .filter(|file : &FileConfig| exists_diff(file, current))
    .collect::<Vec<FileConfig>>()
}

fn exists_diff(file : &FileConfig, current : &Index) -> bool {
  match current.files.iter().find( |x| {
    x.name == file.name
  }) {
    None => true,
    Some(currentfile) => currentfile.md5 != file.md5
  }
}


fn execute(config : &CommandConfig) -> Result<()> {
  print!("Executing\n{}", &config.command);
  for arg in &config.args {
    print!(" {} ", arg);
  }
  println!("");
  let mut command = process::Command::new(&config.command);
  if let &Some(ref env) = &config.env {
    for (key, value) in env {
      println!("env {} {}", &key, &value);
      command.env(key, value);
    }
  }
  command.args(&config.args)
    .spawn().map( |_| { () })
}


pub fn download<F>(source : &str, path_str : &str, expected_md5_opt : Option<&str>, update_progress : F) -> Result<u64>
  where F : FnMut(u64) {
  let client = Client::new();
  let mut res : hyper::client::Response = try!(to_io_result(client.get(source).send()));
  assert_eq!(res.status, hyper::Ok);
  let path = Path::new(&path_str);
  let path_tmp_str = path_str.to_string() + ".tmp";
  let path_tmp = Path::new(&path_tmp_str);
  try!(fs::create_dir_all(path.parent().expect("Failed to get parent")));
  let mut target = try!(File::create(path_tmp));
  let (length, md5) = try!(copy(&mut res, &mut target, update_progress));
  match expected_md5_opt {
    Some(expected_md5) if md5 != expected_md5 =>
      Err(Error::new(ErrorKind::InvalidData, format!("Md5 mismatch: expecting {} got {}", expected_md5, md5))),
    _ => {
      try!(fs::rename(path_tmp, path));
      Ok(length)
    }
  } 
}

fn to_io_result<A>(hyper_result : hyper::error::Result<A>) -> Result<A> {
  hyper_result.map_err(|e| {
    Error::new(ErrorKind::Other, e.to_string())
  })
}

fn copy<R: ?Sized, W: ?Sized, F>(reader: &mut R, writer: &mut W, mut update_progress : F) -> Result<(u64, String)>
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
    try!(writer.write_all(b));
    md5.input(b);
    written += len as u64;
    update_progress(written);
  }
}
