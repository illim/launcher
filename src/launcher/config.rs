extern crate rustc_serialize;
extern crate toml;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::env::consts::{ARCH, OS};
use std::fs;
use rustc_serialize::Decodable;
use launcher::command::CommandConfig;
use launcher::error::*;
use launcher::utils;

#[derive(RustcDecodable)]
pub struct IndexConfig {
  pub directory : String,
  pub file      : String,
  pub source    : String
}

impl IndexConfig {
  pub fn tmpfile(&self) -> String {
    self.file.to_owned() + ".tmp"
  }

  pub fn replace_index(&self) -> io::Result<()> {
    let tmpfile  = self.tmpfile();
    let tmp_path = Path::new(&tmpfile);

    if tmp_path.exists() {
      try!(fs::rename(tmp_path, Path::new(&self.file)));
    }
    Ok(())
  }

  pub fn relativize(&self, file : &FileConfig) -> String {
    self.directory.to_owned() + "/files/" + &file.md5 + "-" + &file.name
  }
}

#[derive(RustcDecodable, Debug)]
pub struct FileConfig {
  pub name   : String,
  pub md5    : String,
  pub source : String,
  pub size   : u64,
  pub action : Option<String>,
  pub os     : Option<String>,
  pub arch   : Option<String>
}

impl FileConfig {

  fn is_current_arch_os(&self) -> bool {
    let os_ok = match self.os {
      Some(ref os) => os == OS,
      _ => true
    };
    let arch_ok = match self.arch {
      Some(ref arch) => arch == ARCH,
      _ => true
    };
    os_ok && arch_ok
  }
}

#[derive(RustcDecodable)]
pub struct Index {
  pub command : CommandConfig,
  pub files   : Vec<FileConfig>
}


pub fn load_index_config() -> BasicResult<IndexConfig> {
  let str = include_str!("../../application.toml");
  deserialize_toml(&inject_vars(&str))
}

pub fn load_index(index_config : &IndexConfig) -> BasicResult<Option<Index>> {
  if Path::new(&index_config.file).exists() {
    let index = try!(read_index(&index_config.file));
    Ok(Some(index))
  } else {
    Ok(None)
  }
}

pub fn read_index(path : &str) -> BasicResult<Index> {
  let mut f = try!(File::open(path));
  let mut s = String::new();
  try!(f.read_to_string(&mut s));
  let index : Index = try!(deserialize_toml(&inject_vars(&s)));
  let index_filtered = Index {
    files : index.files.into_iter().filter(|file| file.is_current_arch_os()).collect() , 
    .. index
  };
  Ok(index_filtered)
}

fn deserialize_toml<T : Decodable>(text : &str) -> BasicResult<T> {
  let mut parser = toml::Parser::new(&text);
  let table = parser.parse();
  let value = match table {
    Some(t) => toml::Value::Table(t),
    None => return Err(From::from(BasicError {
      description : format!("Error while parsing {}", &text),
      errs : parser.errors.into_iter().map(|e| From::from(e)).collect()
    }))
  };
  match toml::decode(value) {
    Some(t) => Ok(t),
    None => Err(From::from(format!("Error while deserializing {}", &text)))
  }
}

fn inject_vars(text : &str) -> String {
  text.replace("${user_home}", &utils::get_home_dir())
}
