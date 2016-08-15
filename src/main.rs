extern crate rustc_serialize;
extern crate hyper;
extern crate zip;
extern crate crypto;

mod launcher;

fn main() {
  let index_config = launcher::config::load_index_config();
  match launcher::app::get_index_state(&index_config) {
    Err(_) => {
      if let Ok(index_opt) = launcher::config::load_index(&index_config) {
        if let Some(index) = index_opt {
          check_launch_result(launcher::app::execute_and_die(&index.command));
        }
      }      
    },
    Ok(index_state) => {
      if index_state.has_diffs() {
        check_launch_result(launcher::ui::process_update(index_config, index_state));
      } else {
        println!("Everything is up to date.");
        if let Some(index) = index_state.current {
          check_launch_result(launcher::app::execute_and_die(&index.command));
        }
      }
    }
  }
}

fn check_launch_result<A>(launch_result : std::io::Result<A>) {
  if let Err(e) = launch_result {
    println!("Failed to launch {}", e);
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
  }
}

