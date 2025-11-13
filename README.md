# Neur

A minimalist static site generator using the [Tera templating engine](https://keats.github.io/tera/), [Lightning CSS](https://lightningcss.dev/), and [Comrak](https://github.com/kivikakk/comrak).

# Documentation

## Installation

### From source

1. Clone the repository

```sh
 git clone https://github.com/aryaveersr/neur
```

2. Install using `cargo install`

```sh
cargo install --path .
```

## Basic usage

Neur comes configured out-of-the-box, allowing you to get started quickly.
To create a new project, simply create a `src` sub-directory for your files and run `neur`.

```
src/
├─ home.css
├─ index.html
├─ about.html
```

The output files are generated in the `dist` sub-directory by default.
To change the default source and output directories, refer to [configuration](#configuration).

## Templates

Neur uses the [Tera templating engine](https://keats.github.io/tera/). All html files starting with an `_` are not considered for generation and thus can be used as layouts, components, etc. Refer to [Tera's documentation](https://keats.github.io/tera/docs/) for more info.

## Markdown

Neur supports generating html pages from markdown files. These files use the `_template.html` found in the same directory, or a [minimal fallback template](src/default.html).

Here's an example of how you might create a custom template:

```
posts/
├─ _template.html
├─ hello-world.md
```

(in \_template.html)

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{{ title }}</title>
  </head>
  <body>
    <header>
      <h1>{{ title }}</h1>
      <span>Published on {{ date }}.</span>
    </header>
    <main>{{ content }}</main>
  </body>
</html>
```

(in hello-world.md)

```markdown
---
title: Hello world!
date: 24th of December.
---

# Hello, world?
```

## CSS

Neur supports CSS syntax lowering using [Lightning CSS](https://lightningcss.dev/), allowing you to use features such as nested styles, logical properties, etc.

## Configuration

Neur comes configured out-of-the-box with sensible defaults, however these can be changed either by passing arguments to the command-line, or defining them in `neur.toml`. In case of conflicts, command-line arguments take precedence.

### Command-line

```sh
neur --help # Show help.
neur --source "pages" --output "out" --minify false
```

### `neur.toml`

```toml
source = "pages"
minify = true
```

### Configuration reference

- source: <directory>. Defaults to "src"
  The path to the source directory to use as input for the generator.

- output: <directory>. Defaults to "dist"
  The path to the output directory to store the generated files.

- minify: bool. Defaults to false.
  Whether to minify the output files.
