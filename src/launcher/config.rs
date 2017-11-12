extern crate toml;

use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::env::consts::{ARCH, OS};
use std::fs;
use launcher::command::CommandConfig;
use launcher::utils;
use errors::*;
use serde::Deserialize;

#[derive(Deserialize)]
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
      fs::rename(tmp_path, Path::new(&self.file))?;
    }
    Ok(())
  }

  pub fn relativize(&self, file : &FileConfig) -> String {
    self.directory.to_owned() + "/files/" + &file.md5 + "-" + &file.name
  }
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize)]
pub struct Index {
  pub command : CommandConfig,
  pub files   : Vec<FileConfig>
}


pub fn load_index_config() -> Result<IndexConfig> {
  let str = include_str!("../../application.toml");
  deserialize_toml(&inject_vars(&str))
}

pub fn load_index(index_config : &IndexConfig) -> Result<Option<Index>> {
  if Path::new(&index_config.file).exists() {
    let index = read_index(&index_config.file)?;
    Ok(Some(index))
  } else {
    Ok(None)
  }
}

pub fn read_index(path : &str) -> Result<Index> {
  let mut f = File::open(path)?;
  let mut s = String::new();
  f.read_to_string(&mut s)?;
  let index : Index = deserialize_toml(&inject_vars(&s))?;
  let index_filtered = Index {
    files : index.files.into_iter().filter(|file| file.is_current_arch_os()).collect() , 
    .. index
  };
  Ok(index_filtered)
}

fn deserialize_toml<'de, T>(text : &'de str) -> Result<T>
  where
    T: Deserialize<'de> {
  toml::from_str(text)
    .chain_err(|| format!("Error while deserializing {}", &text))
}

fn inject_vars(text : &str) -> String {
  text.replace("${user_home}", &utils::get_home_dir())
}
