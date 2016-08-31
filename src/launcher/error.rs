use std::error::Error;
use std::fmt::{self, Display};

pub type BasicResult<A> = Result<A, Box<Error>>;

#[derive(Debug)]
pub struct BasicError {
  pub description : String,
  pub errs : Vec<Box<Error>>
}

impl Display for BasicError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    try!(write!(f, "{}", self.description));
    for err in self.errs.iter() {
      try!(write!(f, "{}", err));
    }
    Ok(())
  }
}

impl Error for BasicError {
  fn description(&self) -> &str {
    self.description.as_str()
  }

  fn cause(&self) -> Option<&Error> {
    self.errs.first().map(|b| b.as_ref())
  }
}

pub fn box_err<A, B : Display>(x : Result<A, B>) -> BasicResult<A> {
  x.map_err(|err| From::from(err.to_string()))
}