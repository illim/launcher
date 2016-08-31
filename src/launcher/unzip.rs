extern crate zip;

use std::error::Error;
use std::io;
use std::fs;
use std::path::Path;
use launcher::error::BasicResult;

pub fn unzip(path : &Path) -> BasicResult<()> {
  let file        = try!(fs::File::open(path));
  let mut archive = try!(zip::ZipArchive::new(file));
  let parent      = try!(path.parent().ok_or::<Box<Error>>(From::from(format!("No parent found for {}", &path.to_string_lossy()))));
  
  for i in 0..archive.len() {
    let mut file = try!(archive.by_index(i));
    let mut pathbuf = parent.to_path_buf();
    pathbuf.push(file.name());
    let outpath = pathbuf.as_path();

    if (file.name()).ends_with("/") {
      try!(fs::create_dir_all(outpath));
    } else {
      try!(fs::create_dir_all(parent));
      let mut outfile = try!(fs::File::create(&outpath));
      try!(io::copy(&mut file, &mut outfile));
    }
  }
  Ok(())
}
