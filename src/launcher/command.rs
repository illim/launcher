use std::collections::HashMap;
use std::io;
use std::process;
use std::path::Path;

#[derive(Deserialize)]
pub struct CommandConfig {
  pub command : String,
  pub args    : Vec<String>,
  pub env     : Option<HashMap<String, String>>
}

impl CommandConfig {

  pub fn execute_and_die(&self) -> io::Result<()> {
    self.execute()?;
    process::exit(0);
  }
    
  fn execute(&self) -> io::Result<()> {
    debug!("Executing {}", &self.command);
    for arg in &self.args {
      debug!(" {} ", arg);
    }
    let mut command = process::Command::new(&self.command);
    if let &Some(ref env) = &self.env {
      for (key, value) in env {
        debug!("env {} {}", &key, &value);
        // hack
        if key.contains("PATH") {
          if Path::new(&value).exists() {
            command.env(key, value);
          }
        } else {
          command.env(key, value);
        }
      }
    }
    command
      .args(&self.args)
      .spawn()
      .map( |_| { () })
  }

}