extern crate zip;

use std::io::{self, Result};
use std::fs;
use std::path::Path;

pub fn unzip(path : &Path) -> Result<()> {
  let file = fs::File::open(path).expect(&format!("Unzip failed open file {}", path.to_string_lossy()));
  let mut archive = zip::ZipArchive::new(file).expect(&format!("Unzip failed open archive {}", path.to_string_lossy()));
  let parent = path.parent().expect(&format!("Unzip failed get parent {}", path.to_string_lossy()));
  
  for i in 0..archive.len() {
    let mut file = archive.by_index(i).expect(&format!("Unzip failed get file archive at {}", i));
    let mut pathbuf = parent.to_path_buf();
    pathbuf.push(file.name());
    let outpath = pathbuf.as_path();

    if (file.name()).ends_with("/") {
      try!(fs::create_dir_all(outpath));
    } else {
      try!(fs::create_dir_all(parent));
      let mut outfile = fs::File::create(&outpath).expect(&format!("Unzip failed create zip output {}", &outpath.to_string_lossy()));
      try!(io::copy(&mut file, &mut outfile));
    }
  }
  Ok(())
}
