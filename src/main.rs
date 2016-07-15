extern crate rustc_serialize;
extern crate ui as libui;
extern crate hyper;
extern crate zip;
extern crate crypto;

mod launcher;

use std::sync::Arc;
use libui::InitOptions;

fn main() {
  let index_config = launcher::config::load_index_config();
  match launcher::app::get_index_state(&index_config) {
    Err(_) => {
      if let Ok(index_opt) = launcher::config::load_index(&index_config) {
        if let Some(index) = index_opt {
          launcher::app::execute_and_die(&index.command);
        }
      }      
    },
    Ok(index_state) => {
      if index_state.has_diffs() {
        libui::init(InitOptions).unwrap();
        let gui = Arc::new(launcher::ui::init_display());
        let handle = launcher::ui::process_update(index_config, index_state, gui.clone());
        launcher::ui::display(gui.clone());
        libui::uninit();
        handle.join().unwrap();
      } else {
        println!("Everything is up to date.");
        if let Some(index) = index_state.current {
          launcher::app::execute_and_die(&index.command)
        }
      }
    }
  }
}


