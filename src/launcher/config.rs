extern crate rustc_serialize;
extern crate toml;

use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::process;
use std::env::consts::{ARCH, OS};
use std::collections::HashMap;
use rustc_serialize::Decodable;

#[derive(RustcDecodable)]
pub struct CommandConfig {
  pub command : String,
  pub args    : Vec<String>,
  pub env     : Option<HashMap<String, String>>
}

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
}

#[derive(RustcDecodable)]
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


pub fn load_index_config() -> IndexConfig {
  let str = include_str!("../../application.toml");
  deserialize_toml(&inject_vars(&str))
}

pub fn load_index(index_config : &IndexConfig) -> Result<Option<Index>, io::Error> {
  if Path::new(&index_config.file).exists() {
    let index = try!(read_index(&index_config.file));
    let index_filtered = Index {
      files : index.files.into_iter().filter(|file| file.is_current_arch_os()).collect() , 
      .. index
    };
    Ok(Some(index_filtered))
  } else {
    Ok(None)
  }
}

pub fn read_index(path : &str) -> Result<Index, io::Error> {
  let mut f = try!(File::open(path));
  let mut s = String::new();
  try!(f.read_to_string(&mut s));
  Ok(deserialize_toml(&inject_vars(&s)))
}


fn get_home_dir() -> String {
  let home_dir = env::home_dir().unwrap_or_else(|| {
    println!("Impossible to get your home dir!");
    process::exit(1);
  });
  format!("{}",home_dir.display()).replace("\\", "\\\\")
}

fn deserialize_toml<T : Decodable>(text : &str) -> T {
  let table = toml::Parser::new(&text).parse();
  let value = match table {
    Some(t) => toml::Value::Table(t),
    None => panic!(format!("Error while parsing {}", &text))
  };
  match toml::decode(value) {
    Some(t) => t,
    None => panic!("Error while deserializing {}", &text)
  }
}

fn inject_vars(text : &str) -> String {
  text.replace("${user_home}", &get_home_dir())
}
