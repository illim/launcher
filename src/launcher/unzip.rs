extern crate zip;

use std::io::{self, Result};
use std::fs;
use std::path::Path;

pub fn unzip(path : &Path) -> Result<()> {
  let file = fs::File::open(path).unwrap();
  let mut archive = zip::ZipArchive::new(file).unwrap();
  let parent = path.parent().unwrap();
  
  for i in 0..archive.len() {
    let mut file = archive.by_index(i).unwrap();
    let mut pathbuf = parent.to_path_buf();
    pathbuf.push(file.name());
    let outpath = pathbuf.as_path();

    if (file.name()).ends_with("/") {
      try!(fs::create_dir_all(outpath));
    } else {
      try!(fs::create_dir_all(parent));
      let mut outfile = fs::File::create(&outpath).unwrap();
      try!(io::copy(&mut file, &mut outfile));
    }
  }
  Ok(())
}
