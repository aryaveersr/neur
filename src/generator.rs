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
use tera::{Context, Tera};

pub type Result<T> = std::result::Result<T, GeneratorError>;

pub struct Generator {
    config: Config,
    tera: Tera,
}

impl Generator {
    pub fn new(config: Config) -> Result<Self> {
        let mut source_glob = config.source.clone();
        source_glob.push("**");
        source_glob.push("*");

        Ok(Self {
            config,
            tera: Tera::new(source_glob.to_str().unwrap())?,
        })
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
            "md" => self.markdown(path)?,
            "html" => self.html(path)?,

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

    fn html(&mut self, path: &Path) -> Result<()> {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let is_escaped = filename.starts_with("_") && !filename.starts_with("__");

        if !is_escaped {
            let trimmed_path = path.components().skip(1).collect::<PathBuf>();
            let rendered = self
                .tera
                .render(trimmed_path.to_str().unwrap(), &Context::new())?;

            fs::write(self.dest(path), rendered)?;
        }

        Ok(())
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
    Io(std::io::Error),
    Css(String),
    Html(tera::Error),
}

impl From<std::io::Error> for GeneratorError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl<T: Display> From<lightningcss::error::Error<T>> for GeneratorError {
    fn from(value: lightningcss::error::Error<T>) -> Self {
        Self::Css(value.to_string())
    }
}

impl From<tera::Error> for GeneratorError {
    fn from(value: tera::Error) -> Self {
        Self::Html(value)
    }
}
