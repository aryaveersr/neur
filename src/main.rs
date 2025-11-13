use neur::{Config, ConfigError, Generator, GeneratorError};
use std::fmt::Debug;

fn main() -> Result<(), Error> {
    Generator::new(Config::parse()?)?.run()?;
    Ok(())
}

enum Error {
    Config(ConfigError),
    Generator(GeneratorError),
}

impl From<ConfigError> for Error {
    fn from(value: ConfigError) -> Self {
        Error::Config(value)
    }
}

impl From<GeneratorError> for Error {
    fn from(value: GeneratorError) -> Self {
        Error::Generator(value)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(err) => {
                writeln!(f, "While loading the configuration:")?;
                write!(f, "{err:?}")
            }

            Self::Generator(err) => {
                writeln!(f, "While generating the site:")?;
                write!(f, "{err:?}")
            }
        }
    }
}
