use crate::Config;
use lightningcss::{printer::PrinterOptions, stylesheet::StyleSheet};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use tera::{Context, Tera};

pub type Result<T> = std::result::Result<T, GeneratorError>;

pub struct Generator {
    config: Config,
    tera: Tera,
    templates: Vec<PathBuf>,
}

impl Generator {
    pub fn new(config: Config) -> Result<Self> {
        let source_glob = config.source.join("**/*");

        Ok(Self {
            config,
            templates: Vec::new(),
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

        let mut markdown_paths = Vec::new();

        match extension {
            "css" => self.css(path)?,
            "html" => self.html(path)?,
            "md" => markdown_paths.push(path.to_path_buf()),

            _ => {
                fs::copy(path, self.dest(path))?;
            }
        };

        for path in markdown_paths {
            self.markdown(&path)?;
        }

        Ok(())
    }

    fn css(&mut self, path: &Path) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        let mut styles = StyleSheet::parse(&contents, Default::default())?;

        let printer_opts = PrinterOptions {
            minify: self.config.minify,
            ..Default::default()
        };

        styles.minify(Default::default())?;
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

            if self.config.minify {
                fs::write(
                    self.dest(path),
                    minify_html::minify(rendered.as_bytes(), &Default::default()),
                )?;
            } else {
                fs::write(self.dest(path), rendered)?;
            }
        } else if filename == "_template.html" {
            self.templates.push(path.parent().unwrap().into());
        }

        Ok(())
    }

    fn markdown(&mut self, path: &Path) -> Result<()> {
        let contents = fs::read_to_string(path)?;
        let mut context = Context::new();

        let mut options = comrak::Options::default();
        options.extension.front_matter_delimiter = Some("---".into());

        context.try_insert("content", &comrak::markdown_to_html(&contents, &options))?;

        let (props, _): (HashMap<String, toml::Value>, _) = markdown_frontmatter::parse(&contents)?;

        for (key, value) in props {
            context.try_insert(key, &value)?;
        }

        let rendered = if self.templates.contains(&path.parent().unwrap().into()) {
            let trimmed_path = path
                .parent()
                .unwrap()
                .join("_template.html")
                .components()
                .skip(1)
                .collect::<PathBuf>();

            self.tera.autoescape_on(vec![]);
            let rendered = self.tera.render(trimmed_path.to_str().unwrap(), &context)?;
            self.tera.autoescape_on(vec![".html", ".htm", ".xml"]);

            rendered
        } else {
            Tera::one_off(include_str!("default.html"), &context, false)?
        };

        let dest = self.dest(path).with_extension("html");

        if self.config.minify {
            fs::write(
                dest,
                minify_html::minify(rendered.as_bytes(), &Default::default()),
            )?;
        } else {
            fs::write(dest, rendered)?;
        }

        Ok(())
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
    Toml(toml::de::Error),
    Frontmatter(markdown_frontmatter::Error),
}

impl From<std::io::Error> for GeneratorError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl<T: std::fmt::Display> From<lightningcss::error::Error<T>> for GeneratorError {
    fn from(value: lightningcss::error::Error<T>) -> Self {
        Self::Css(value.to_string())
    }
}

impl From<tera::Error> for GeneratorError {
    fn from(value: tera::Error) -> Self {
        Self::Html(value)
    }
}

impl From<toml::de::Error> for GeneratorError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

impl From<markdown_frontmatter::Error> for GeneratorError {
    fn from(value: markdown_frontmatter::Error) -> Self {
        Self::Frontmatter(value)
    }
}
