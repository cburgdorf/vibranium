extern crate vibranium;
extern crate toml;

use std::error::Error;
use std::fmt;

use vibranium::compiler::error::CompilerError;

#[derive(Debug)]
pub enum CliError {
  CompilationError(CompilerError),
  ConfigurationSetError(toml::ser::Error),
}

impl Error for CliError {
  fn description(&self) -> &str {
    match self {
      CliError::CompilationError(error) => {
        match error {
          CompilerError::UnsupportedStrategy => r###"No built-in support for requested compiler.
To use this compiler, please specify necessary OPTIONS in compile command. E.g:

  vibranium compile --compiler solcjs -- <OPTIONS>...

OPTIONS can also be specified in the project's vibranium.toml file:

  [compiler]
    options = ["--option1", "--option2"]
"###,
          _ => error.description()

        }
      },
      CliError::ConfigurationSetError(error) => error.description(),
    }
  }

  fn cause(&self) -> Option<&Error> {
    match self {
      CliError::CompilationError(error) => Some(error),
      CliError::ConfigurationSetError(error) => Some(error),
    }
  }
}

impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CliError::CompilationError(_error) => write!(f, "{}", self.description()),
      CliError::ConfigurationSetError(error) => write!(f, "Couldn't set configuration: {}", error),
    }
  }
}

