
use std::io::{Result, Write, stderr, stdout};
use std::path::Path;
use std::cell::Cell;
use launcher::config::{IndexConfig, FileConfig};
use launcher::app::{self, IndexState};
use launcher::unzip;


pub fn process_update(index_config : IndexConfig, index_state : IndexState) -> Result<()> {
  let command_config = index_state.index.command;
  let files = index_state.index.files;
  let files_to_update = match index_state.current {
    Some(current) => app::filter_diffs(files, &current),
    None => files
  };

  try!(update_files(&index_config, &files_to_update));
  app::replace_index(&index_config);
  app::execute_and_die(&command_config)
}

fn update_files(index_config : &IndexConfig, files : &Vec<FileConfig>) -> Result<()> {
  for (i, file) in files.iter().enumerate() {
    let target_str = index_config.directory.to_owned() + "/files/" + &file.md5 + "-" + &file.name;
    let target = Path::new(&target_str);

    if target.exists() {
      try!(exec_action(&file, &target));
    } else {
      print!("[{}/{}] Downloading {} ", i + 1, files.len(), file.name);
      let _ = stdout().flush();
      let current_progress : Cell<u64> = Cell::new(0);
      let download_result = app::download(&file.source, &target_str, Some(&file.md5), |nb_byte_read| {
        let progress = nb_byte_read * 100 / file.size;
        if progress % 10 == 0 && progress > current_progress.get() {
          print!("#");
          let _ = stdout().flush();
          current_progress.set(progress);
        }
      });

      match download_result {
        Ok(_) => {
          println!(" Done");
          try!(exec_action(&file, &target));
        },
        Err(e) => {
          let stderr = stderr();
          let mut err = stderr.lock();
          try!(write!(err, "Failed download from {} : {}", file.source, e));
          return Err(e);
        }
      }
    }
  }
  Ok(())
}

fn exec_action(file : &FileConfig, path : &Path) -> Result<()>{
  match file.action {
    Some(ref action) if action == "unzip" => {
      println!("Unzipping {}", file.name);
      unzip::unzip(path)
    },
    _ => Ok(())
  }
}


