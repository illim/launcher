extern crate reqwest;

use launcher::config::{self, IndexConfig, FileConfig, Index};
use errors::*;
use launcher::utils;

pub struct IndexState {
  pub current : Option<Index>,
  pub index : Index
}

pub fn get_index_state(index_config : &IndexConfig) -> Result<IndexState> {
  let tmpfile= index_config.tmpfile();

  utils::download(&index_config.source, &tmpfile, None, |_| {} )?;
  Ok(
    IndexState {
      current : config::load_index(index_config)?,
      index   : config::read_index(&index_config.tmpfile())?
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
    .collect()
}

fn exists_diff(file : &FileConfig, current : &Index) -> bool {
  match current.files.iter().find( |x| {
    x.name == file.name
  }) {
    None => true,
    Some(currentfile) => currentfile.md5 != file.md5
  }
}

pub fn get_outdated_files(index_config: &IndexConfig, reffiles : &Vec<FileConfig>, current : &Index) -> Vec<String> {
  current.files
    .iter()
    .filter_map(|ref file| {
      if ! reffiles.iter().any(|f| f.name == file.name) {
        Some(index_config.relativize(file))
      } else {
        None
      }
    })
    .collect()
}