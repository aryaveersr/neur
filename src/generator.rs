use crate::Config;
use lightningcss::{
    printer::PrinterOptions,
    stylesheet::StyleSheet,
    targets::{Browsers, Targets},
};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
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
        let mut styles =
            StyleSheet::parse(&contents, Default::default()).map_err(|err| (path, err))?;

        styles
            .minify(Default::default())
            .map_err(|err| (path, err))?;

        let output = styles
            .to_css(PrinterOptions {
                minify: self.config.minify,
                targets: Targets {
                    browsers: Some(
                        Browsers::from_browserslist(std::iter::once("last 4 years"))
                            .unwrap()
                            .unwrap(),
                    ),
                    ..Default::default()
                },
                ..Default::default()
            })
            .map_err(|err| (path, err))?;

        fs::write(self.dest(path), output.code)?;

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

        let (props, _): (HashMap<String, toml::Value>, _) =
            markdown_frontmatter::parse(&contents).map_err(|err| (path, err))?;

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

pub enum GeneratorError {
    Io(std::io::Error),
    Tera(tera::Error),

    Css {
        file: PathBuf,
        err: String,
    },

    Frontmatter {
        file: PathBuf,
        err: markdown_frontmatter::Error,
    },
}

impl From<std::io::Error> for GeneratorError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<tera::Error> for GeneratorError {
    fn from(value: tera::Error) -> Self {
        Self::Tera(value)
    }
}

impl<T: Display> From<(&Path, lightningcss::error::Error<T>)> for GeneratorError {
    fn from((file, err): (&Path, lightningcss::error::Error<T>)) -> Self {
        Self::Css {
            file: file.to_path_buf(),
            err: err.to_string(),
        }
    }
}

impl From<(&Path, markdown_frontmatter::Error)> for GeneratorError {
    fn from((file, err): (&Path, markdown_frontmatter::Error)) -> Self {
        Self::Frontmatter {
            file: file.to_path_buf(),
            err,
        }
    }
}

impl Debug for GeneratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),

            Self::Css { file, err } => {
                writeln!(f, "While parsing the css at {}:", file.display())?;
                write!(f, "{err}")
            }

            Self::Tera(err) => {
                writeln!(f, "While generating tera template:")?;
                write!(f, "{err}")
            }

            Self::Frontmatter { file, err } => {
                writeln!(f, "While parsing the frontmatter from {}:", file.display())?;
                write!(f, "{err}")
            }
        }
    }
}
