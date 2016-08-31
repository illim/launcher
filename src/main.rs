extern crate rustc_serialize;
extern crate hyper;
extern crate zip;
extern crate crypto;
#[macro_use] extern crate log;
extern crate env_logger;

mod launcher;

use std::env;
use launcher::error::*;
use log::LogLevelFilter;
use env_logger::LogBuilder;

fn main() {
  init_logger();
  if let Err(e) = run() {
    error!("Failed to launch {}", e);
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
  }
}

fn run() -> BasicResult<()> {
  let index_config = try!(launcher::config::load_index_config());
  match launcher::state::get_index_state(&index_config) {
    Err(err) => {
      error!("Failed getting index caused by : {}", err);
      if let Ok(index_opt) = launcher::config::load_index(&index_config) {
        if let Some(index) = index_opt {
          try!(index.command.execute_and_die());
        }
      }
    },
    Ok(index_state) => {
      let command = if index_state.has_diffs() {
        try!(launcher::update::process_update(&index_config, index_state))
      } else {
        info!("Everything is up to date.");
        index_state.index.command
      };
      try!(command.execute_and_die());
    }
  }
  Ok(())
}


fn init_logger() {
  let mut builder = LogBuilder::new();

  builder
    .filter(None, LogLevelFilter::Info);

  if env::var("RUST_LOG").is_ok() {
    builder.parse(&env::var("RUST_LOG").unwrap());
  }

  builder.init().unwrap();
}