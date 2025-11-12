use clap::Parser;
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct Config {
    pub source: PathBuf,
    pub output: PathBuf,
    pub minify: bool,
}

impl Config {
    pub fn parse() -> Result<Self, ConfigError> {
        Ok(Options::new()?.try_into()?)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    ParseError(toml::de::Error),
    IoError(std::io::Error),
    ValidationError(String),
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self::ParseError(value)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Parser, Clone, Deserialize)]
struct Options {
    #[arg(short, long, value_name = "DIR")]
    source: Option<PathBuf>,

    #[arg(short, long, value_name = "DIR")]
    output: Option<PathBuf>,

    #[arg(skip)]
    minify: Option<bool>,

    #[serde(skip)]
    #[arg(short, long = "minify")]
    _minify_cli: Option<Option<bool>>,

    #[serde(skip)]
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

impl Options {
    fn new() -> Result<Self, ConfigError> {
        let mut cli_opts = Self::parse();
        let path = match &cli_opts.config {
            Some(path) => Some(path.to_owned()),
            None => {
                let path = PathBuf::from("neur.toml");
                if path.try_exists()? { Some(path) } else { None }
            }
        };

        if let Some(path) = path {
            let contents = fs::read_to_string(path)?;
            cli_opts = cli_opts.merge(toml::from_str(&contents)?);
        }

        Ok(cli_opts)
    }

    fn merge(mut self, rhs: Self) -> Self {
        self.source = self.source.or(rhs.source);
        self.output = self.output.or(rhs.output);
        self.minify = self._minify_cli.map(|o| o.unwrap_or(true)).or(rhs.minify);
        self
    }
}

impl TryFrom<Options> for Config {
    type Error = ConfigError;

    fn try_from(opts: Options) -> Result<Self, Self::Error> {
        let cfg = Config {
            source: opts.source.unwrap_or("src".into()),
            output: opts.output.unwrap_or("dist".into()),
            minify: opts.minify.unwrap_or(false),
        };

        if cfg.output.starts_with(&cfg.source) {
            return Err(ConfigError::ValidationError(
                "Output directory can't be under source directory.".into(),
            ));
        } else if cfg.source.starts_with(&cfg.output) {
            return Err(ConfigError::ValidationError(
                "Source directory can't be under output directory.".into(),
            ));
        }

        Ok(cfg)
    }
}
