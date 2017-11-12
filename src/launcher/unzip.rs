extern crate zip;

use std::io;
use std::fs;
use std::path::Path;
use errors::*;

pub fn unzip(path : &Path) -> Result<()> {
  let file        = fs::File::open(path)?;
  let mut archive = zip::ZipArchive::new(file)?;
  let parent      = path.parent().ok_or(format!("No parent found for {}", &path.to_string_lossy()))?;
  
  for i in 0..archive.len() {
    let mut file = archive.by_index(i)?;
    let mut pathbuf = parent.to_path_buf();
    pathbuf.push(file.name());
    let outpath = pathbuf.as_path();

    if (file.name()).ends_with("/") {
      fs::create_dir_all(outpath)?;
    } else {
      fs::create_dir_all(parent)?;
      let mut outfile = fs::File::create(&outpath)?;
      io::copy(&mut file, &mut outfile)?;
    }
  }
  Ok(())
}
