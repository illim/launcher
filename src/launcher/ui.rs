extern crate ui as libui;

use std::io::Result;
use std::path::Path;
use std::thread;
use std::cell::Cell;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use libui::{BoxControl, Window, MultilineEntry, ProgressBar};
use launcher::config::{IndexConfig, FileConfig};
use launcher::app::{self, IndexState};
use launcher::unzip;


pub struct Gui {
  logs : Mutex<MultilineEntry>,
  progress : Mutex<ProgressBar>
}

pub fn init_display() -> Gui {
  Gui {
    logs : Mutex::new(MultilineEntry::new()),
    progress : Mutex::new(ProgressBar::new())
  }
}

pub fn process_update(index_config : IndexConfig, index_state : IndexState, gui : Arc<Gui>) -> JoinHandle<()> {
  let gui = gui.clone();
  thread::spawn(move || {
    let command_config = index_state.index.command;
    let reffiles = index_state.index.files;
    let files = match index_state.current {
      Some(current) => app::filter_diffs(reffiles, &current),
      None => reffiles
    };

    if let Err(err) = update_files(&index_config, &files, gui) {
      panic!("Err updating {}", err)
    }
    app::replace_index(&index_config);
    app::execute_and_die(&command_config);
  })
}

fn update_files(index_config : &IndexConfig, files : &Vec<FileConfig>, gui : Arc<Gui>) -> Result<()> {
  let current_progress : Cell<u64> = Cell::new(0);
  let total_size : u64 = files.iter().fold(0, |acc, file| acc + file.size);
  for file in files {
    let target_str = index_config.directory.to_owned() + "/files/" + &file.name;
    let target = Path::new(&target_str);
    log_msg(format!("downloading {}\n", &file.name), gui.clone());
    let res = app::download(&file.source, &target_str, Some(&file.md5), |progress| {
      update_progress((progress + current_progress.get()) * 100 / total_size, gui.clone());
    });
    match res {
      Ok(progress) => {
        current_progress.set(current_progress.get() + progress);
        log_msg(format!("download {} successful\n", &file.name), gui.clone());
        try!(exec_action(&file, &target, gui.clone()));
      },
      Err(e) => {
        log_msg(format!("Failed download from {} : {}\n", &file.source, e), gui.clone());
        return Err(e);
      }
    }
  }
  Ok(())
}

fn exec_action(file : &FileConfig, path : &Path, gui : Arc<Gui>) -> Result<()>{
  match file.action {
    Some(ref action) if action == "unzip" => {
      log_msg(format!("Unzipping {}\n", &file.name), gui.clone());
      try!(unzip::unzip(path));
    },
    _ => ()
  };
  Ok(())
}

fn log_msg(s : String, gui : Arc<Gui>) {
  libui::queue_main(Box::new(move || {
    if let Ok(logs) = gui.logs.lock() {
      let text = logs.text().to_string() + &s;
      logs.set_text(&text)
    }
  }));
}

fn update_progress(x : u64, gui : Arc<Gui>) {
  if let Ok(progress) = gui.progress.lock() {
    progress.set_value(x as i32);
  }
}

pub fn display(gui : Arc<Gui>) {
  let mainwin = Window::new("Updating...", 350, 200, true);
  mainwin.set_margined(true);
  mainwin.on_closing(Box::new(|_| {
    libui::quit();
    false
  }));

  let vbox = BoxControl::new_vertical();
  mainwin.set_child(vbox.clone().into());

  if let Ok(logs) = gui.logs.lock() {
    vbox.append((*logs).clone().into(), true);
  }

  if let Ok(progress) = gui.progress.lock() {
    vbox.append((*progress).clone().into(), false);
  }
  
  mainwin.show();
  libui::main();
}
