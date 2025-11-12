use crate::Config;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::{MinifyOptions, ParserOptions, StyleSheet},
};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, GeneratorError>;

pub struct Generator {
    config: Config,
}

impl Generator {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run(&mut self) -> Result<()> {
        let source = self.config.source.clone();
        self.directory(&source)
    }

    fn directory(&mut self, path: &Path) -> Result<()> {
        fs::create_dir_all(self.dest(path))?;

        for entry in fs::read_dir(path)? {
            let entry = entry?;

            if entry.file_type()?.is_dir() {
                self.directory(entry.path().as_path())?;
            } else if entry.file_type()?.is_file() {
                self.file(entry.path().as_path())?;
            }
        }

        Ok(())
    }

    fn file(&mut self, path: &Path) -> Result<()> {
        let extension = match path.extension() {
            Some(ext) => ext.to_str().unwrap(),
            None => "",
        };

        match extension {
            "css" => self.css(path)?,
            "html" => self.html(path)?,
            "md" => self.markdown(path)?,
            _ => {
                fs::copy(path, self.dest(path))?;
            }
        };

        Ok(())
    }

    fn css(&mut self, path: &Path) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        let mut styles = StyleSheet::parse(&contents, ParserOptions::default())?;

        let printer_opts = PrinterOptions {
            minify: self.config.minify,
            ..Default::default()
        };

        styles.minify(MinifyOptions::default())?;
        fs::write(self.dest(path), styles.to_css(printer_opts)?.code)?;

        Ok(())
    }

    fn html(&mut self, _path: &Path) -> Result<()> {
        todo!()
    }

    fn markdown(&mut self, _path: &Path) -> Result<()> {
        todo!()
    }

    fn dest(&self, path: &Path) -> PathBuf {
        self.config
            .output
            .join(path.components().skip(1).collect::<PathBuf>())
    }
}

#[derive(Debug)]
pub enum GeneratorError {
    IoError(std::io::Error),
    CssError(String),
}

impl From<std::io::Error> for GeneratorError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl<T: Display> From<lightningcss::error::Error<T>> for GeneratorError {
    fn from(value: lightningcss::error::Error<T>) -> Self {
        Self::CssError(value.to_string())
    }
}
