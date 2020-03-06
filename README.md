# LabNotes

LabNotes is a command-line tool for serving a directory
of [Markdown](https://daringfireball.net/projects/markdown/)
files as a simple website. It is designed for the purpose
of keeping a set of notes for academic research. As such it
supports simple inclusion of mathematical equations,
links to [arXiv](https://arxiv.org/)/[DOI](https://dx.doi.org/)
records, and tables.

It is implemented in Rust, using a [Rocket](https://rocket.rs/)
webserver and the [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
Markdown parser.

## Installation

To build from source, you need a *nightly* version of Rust,
which can be installed using [rustup](https://rustup.rs/).
You should then be able to compile and run LabNotes with
the command
```
cargo +nightly run -- <dir>
```
where `<dir>` is the directory containing your markdown files.
