use maud::{Markup, PreEscaped, Render};
use pulldown_cmark::{
    escape::escape_href, escape::escape_html, html, Alignment, CodeBlockKind, CowStr, Event,
    LinkType, Options, Parser, Tag,
};
use regex::Regex;
use std::collections::HashMap;
use std::io;

/// Renders a block of Markdown using `pulldown-cmark`.
pub struct Markdown<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for Markdown<T> {
    fn render(&self) -> Markup {
        // Compile regular expressions to match arXiv and DOI references
        let new_arxiv = Regex::new(r"^ar[xX]iv:([0-9]{4}[.][0-9]{4,}(v[0-9]+)?)$").unwrap();
        let old_arxiv = Regex::new(r"^(ar[xX]iv:)?([a-zA-Z.-]+/[0-9]{7}(v[0-9]+)?)$").unwrap();
        let doi = Regex::new(r"^(doi:)?(10[.][0-9.]+/[0-9a-zA-Z()._-]+)$").unwrap();

        let mut reference_callback = |link: pulldown_cmark::BrokenLink| {
            let reference = link.reference;
            if let Some(c) = new_arxiv.captures(&reference) {
                Some((
                    format!("https://arxiv.org/abs/{}", c.get(1).unwrap().as_str()),
                    format!("arXiv:{}", c.get(1).unwrap().as_str()),
                ))
            } else if let Some(c) = old_arxiv.captures(&reference) {
                Some((
                    format!("https://arxiv.org/abs/{}", c.get(2).unwrap().as_str()),
                    format!("{}", c.get(2).unwrap().as_str()),
                ))
            } else if let Some(c) = doi.captures(&reference) {
                Some((
                    format!("https://dx.doi.org/{}", c.get(2).unwrap().as_str()),
                    format!("doi:{}", c.get(2).unwrap().as_str()),
                ))
            } else {
                None
            }
            .map(|(url, label)| {
                (url.into(), label.into()) as (pulldown_cmark::CowStr, pulldown_cmark::CowStr)
            })
        };

        // Generate raw HTML
        let parser = Parser::new_with_broken_link_callback(
            self.0.as_ref(),
            Options::all(),
            Some(&mut reference_callback),
        );

        let mut katex = KatexMiddleware::new();
        let parser = parser.filter_map(move |e| katex.map(e));

        let mut unsafe_html = String::new();
        html::push_html(&mut unsafe_html, parser);

        // Sanitize it with ammonia
        //let safe_html = ammonia::clean(&unsafe_html);
        PreEscaped(unsafe_html)
    }
}

struct KatexMiddleware {
    tags: usize,
}

impl KatexMiddleware {
    fn new() -> KatexMiddleware {
        KatexMiddleware { tags: 0 }
    }

    fn map<'a>(&'_ mut self, event: Event<'a>) -> Option<Event<'a>> {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(kind))) => {
                if kind.as_ref() == "math" {
                    self.tags += 1;
                    None
                } else {
                    Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(kind))))
                }
            }
            Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(kind))) => {
                if kind.as_ref() == "math" {
                    self.tags -= 1;
                    None
                } else {
                    Some(Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(kind))))
                }
            }
            Event::Text(text) => {
                if self.tags > 0 {
                    let opts = katex::Opts::builder().display_mode(true).build().unwrap();
                    Some(Event::Html(CowStr::from(
                        katex::render_with_opts(text.as_ref(), opts).unwrap_or_else(|e| match e {
                            katex::Error::JsExecError(s) => {
                                format!("<div class=\"todo\">{}</div>", s)
                            }
                            _ => panic!("{}", e),
                        }),
                    )))
                } else {
                    Some(Event::Text(text))
                }
            }
            Event::Code(code) => {
                let s = code.as_ref();
                if let Some(text) = s.strip_prefix("$").and_then(|s| s.strip_suffix("$")) {
                    Some(Event::Html(CowStr::from(
                        katex::render(text).unwrap_or_else(|e| match e {
                            katex::Error::JsExecError(s) => {
                                format!("<span class=\"todo\">{}</span>", s)
                            }
                            _ => panic!("{}", e),
                        }),
                    )))
                } else {
                    Some(Event::Code(code))
                }
            }
            e => Some(e),
        }
    }
}

impl<T: AsRef<str>> Markdown<T> {
    pub fn render_tex(&self) -> String {
        // Compile regular expressions to match arXiv and DOI references
        let new_arxiv = Regex::new(r"^ar[xX]iv:([0-9]{4}[.][0-9]{4,}(v[0-9]+)?)$").unwrap();
        let old_arxiv = Regex::new(r"^(ar[xX]iv:)?([a-zA-Z.-]+/[0-9]{7}(v[0-9]+)?)$").unwrap();
        let doi = Regex::new(r"^(doi:)?(10[.][0-9.]+/[0-9a-zA-Z()._-]+)$").unwrap();

        let mut reference_callback = |link: pulldown_cmark::BrokenLink| {
            let reference = link.reference;
            if let Some(c) = new_arxiv.captures(&reference) {
                Some((
                    format!("https://arxiv.org/abs/{}", c.get(1).unwrap().as_str()),
                    format!("arXiv:{}", c.get(1).unwrap().as_str()),
                ))
            } else if let Some(c) = old_arxiv.captures(&reference) {
                Some((
                    format!("https://arxiv.org/abs/{}", c.get(2).unwrap().as_str()),
                    format!("{}", c.get(2).unwrap().as_str()),
                ))
            } else if let Some(c) = doi.captures(&reference) {
                Some((
                    format!("https://dx.doi.org/{}", c.get(2).unwrap().as_str()),
                    format!("doi:{}", c.get(2).unwrap().as_str()),
                ))
            } else {
                None
            }
            .map(|(url, label)| {
                (url.into(), label.into()) as (pulldown_cmark::CowStr, pulldown_cmark::CowStr)
            })
        };

        // Generate raw HTML
        let parser = Parser::new_with_broken_link_callback(
            self.0.as_ref(),
            Options::all(),
            Some(&mut reference_callback),
        );

        let mut latex = String::new();
        push_latex(&mut latex, parser);

        latex
    }
}

fn push_latex<'a, I>(s: &mut String, iter: I)
where
    I: Iterator<Item = Event<'a>>,
{
    LatexWriter::new(iter, s).run().unwrap();
}

struct LatexWriter<'a, I, W> {
    /// Iterator supplying events.
    iter: I,

    /// Writer to write to.
    writer: W,

    /// Whether or not the last write wrote a newline.
    end_newline: bool,

    table_cells: usize,
    table_cell_index: usize,
    numbers: HashMap<CowStr<'a>, usize>,
}

impl<'a, I, W> LatexWriter<'a, I, W>
where
    I: Iterator<Item = Event<'a>>,
    W: pulldown_cmark::escape::StrWrite,
{
    fn new(iter: I, writer: W) -> Self {
        Self {
            iter,
            writer,
            end_newline: true,
            table_cells: 0,
            table_cell_index: 0,
            numbers: HashMap::new(),
        }
    }

    /// Writes a new line.
    fn write_newline(&mut self) -> io::Result<()> {
        self.end_newline = true;
        self.writer.write_str("\n")
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    #[inline]
    fn write(&mut self, s: &str) -> io::Result<()> {
        self.writer.write_str(s)?;

        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    pub fn run(mut self) -> io::Result<()> {
        self.write("\\documentclass{article}\n\n")?;
        self.write("\\usepackage[normalem]{ulem}\n")?;
        self.write("\\usepackage{minted}\n")?;
        self.write("\\usepackage{graphicx}\n")?;
        self.write("\\usepackage{hyperref}\n")?;
        self.write("\\usepackage{amsmath}\n\n")?;
        self.write("\\setcounter{tocdepth}{6}\n")?;
        self.write("\\setcounter{secnumdepth}{6}\n\n")?;
        self.write("\\begin{document}\n")?;
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(tag) => {
                    self.start_tag(tag)?;
                }
                Event::End(tag) => {
                    self.end_tag(tag)?;
                }
                Event::Text(text) => {
                    self.write(&text)?;
                    self.end_newline = text.ends_with('\n');
                }
                Event::Code(text) => {
                    if let Some(_) = text
                        .strip_prefix("$")
                        .and_then(|text| text.strip_suffix("$"))
                    {
                        self.write(&text)?
                    } else {
                        self.write(r"\mintinline{text}{")?;
                        self.write(&text)?;
                        self.write(r"}")?;
                    }
                }
                Event::Html(html) => {
                    self.write(r"\begin{verbatim}")?;
                    self.write(&html)?;
                    self.write(r"\end{verbatim}")?;
                }
                Event::SoftBreak => {
                    self.write_newline()?;
                }
                Event::HardBreak => {
                    self.write(r"\\")?;
                    self.write_newline()?;
                }
                Event::Rule => {
                    if self.end_newline {
                        self.write("\\hline\n")?;
                    } else {
                        self.write("\n\\hline\n")?;
                    }
                }
                Event::FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    self.write(r"\footnotemark[")?;
                    let number = *self.numbers.entry(name).or_insert(len);
                    write!(&mut self.writer, "{}", number)?;
                    self.write("]")?;
                }
                Event::TaskListMarker(true) => {
                    self.write(r"\makebox[0pt][l]{$\square$}\raisebox{.15ex}{\hspace{0.1em}$\checkmark$}\n")?;
                }
                Event::TaskListMarker(false) => {
                    self.write(r"\makebox[0pt][l]{$\square$}\n")?;
                }
            }
        }
        self.write("\\end{document}\n")?;
        Ok(())
    }

    /// Writes the start of an HTML tag.
    fn start_tag(&mut self, tag: Tag<'a>) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {
                if self.end_newline {
                    self.write("\n")
                } else {
                    self.write("\n\n")
                }
            }
            Tag::Heading(level, _, _) => {
                use pulldown_cmark::HeadingLevel::{H1, H2, H3, H4, H5, H6};
                let section = match level {
                    H1 => "section*",
                    H2 => "subsection*",
                    H3 => "subsubsection*",
                    H4 => "paragraph",
                    H5 => "subparagraph",
                    H6 => "subsubparagraph",
                };
                if self.end_newline {
                    self.end_newline = false;
                    write!(&mut self.writer, "\\{}{{", section)
                } else {
                    write!(&mut self.writer, "\n{}{{", section)
                }
            }
            Tag::Table(alignments) => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                self.write("\\begin{center}\n\\begin{tabular}{")?;
                for alignment in &alignments {
                    match alignment {
                        &Alignment::Center => self.write("c")?,
                        &Alignment::Right => self.write("r")?,
                        _ => self.write("l")?,
                    }
                }
                self.table_cells = alignments.len();
                self.write("}\n")
            }
            Tag::TableHead => {
                self.table_cell_index = 0;
                Ok(())
            }
            Tag::TableRow => {
                self.table_cell_index = 0;
                Ok(())
            }
            Tag::TableCell => Ok(()),
            Tag::BlockQuote => {
                if self.end_newline {
                    self.write("\\begin{quotation}\n")
                } else {
                    self.write("\n\\begin{quotation}\n")
                }
            }
            Tag::CodeBlock(info) => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                match info {
                    CodeBlockKind::Fenced(info) => {
                        let lang = info.split(' ').next().unwrap();
                        if lang.is_empty() {
                            self.write("\\begin{minted}{text}\n")
                        } else if lang == "math" {
                            self.write("\\begin{align}\n")
                        } else {
                            self.write("\\begin{minted}{")?;
                            escape_html(&mut self.writer, lang)?;
                            self.write("}\n")
                        }
                    }
                    CodeBlockKind::Indented => self.write("\\begin{minted}{text}\n"),
                }
            }
            Tag::List(Some(1)) => {
                if self.end_newline {
                    self.write("\\begin{enumerate}\n")
                } else {
                    self.write("\n\\begin{enumerate}\n")
                }
            }
            Tag::List(Some(start)) => {
                if self.end_newline {
                    self.write("\\begin{enumerate}\n")?;
                } else {
                    self.write("\n\\begin{enumerate}\n")?;
                }
                self.write("\\setcounter{enumi}{")?;
                write!(&mut self.writer, "{}", start)?;
                self.write("}\n")
            }
            Tag::List(None) => {
                if self.end_newline {
                    self.write("\\begin{itemize}\n")
                } else {
                    self.write("\n\\begin{itemize}\n")
                }
            }
            Tag::Item => {
                if self.end_newline {
                    self.write("\\item ")
                } else {
                    self.write("\n\\item ")
                }
            }
            Tag::Emphasis => self.write("{\\em "),
            Tag::Strong => self.write("{\\bf "),
            Tag::Strikethrough => self.write("\\sout{"),
            Tag::Link(LinkType::Email, dest, _title) => {
                self.write("\\href{mailto:")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("}{")
            }
            Tag::Link(_link_type, dest, _title) => {
                self.write("\\href{")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("}{")
            }
            Tag::Image(_link_type, dest, _title) => {
                self.write("\\includegraphics{")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("}\n")?;
                self.consume_text()
            }
            Tag::FootnoteDefinition(name) => {
                if self.end_newline {
                    self.write("\\footnotetext[")?;
                } else {
                    self.write("\n\\footnotetext[")?;
                }
                let len = self.numbers.len() + 1;
                let number = *self.numbers.entry(name).or_insert(len);
                write!(&mut self.writer, "{}", number)?;
                self.write("]{")
            }
        }
    }

    fn end_tag(&mut self, tag: Tag) -> io::Result<()> {
        match tag {
            Tag::Paragraph => {}
            Tag::Heading(_level, _, _) => {
                self.write("}\n")?;
            }
            Tag::Table(_) => {
                self.write("\\end{tabular}\n\\end{center}\n")?;
            }
            Tag::TableHead => {
                self.write("\\hline\n")?;
            }
            Tag::TableRow => {}
            Tag::TableCell => {
                self.table_cell_index += 1;
                if self.table_cell_index == self.table_cells {
                    self.write("\\\\\n")?;
                } else {
                    self.write(" & ")?;
                }
            }
            Tag::BlockQuote => {
                self.write("\\end{quotation}\n")?;
            }
            Tag::CodeBlock(info) => match info {
                CodeBlockKind::Fenced(info) => {
                    let lang = info.split(' ').next().unwrap();
                    if lang == "math" {
                        self.write("\\end{align}\n")?;
                    } else {
                        self.write("\\end{minted}\n")?;
                    }
                }
                CodeBlockKind::Indented => {
                    self.write("\\end{minted}\n")?;
                }
            },
            Tag::List(Some(_)) => {
                self.write("\\end{enumerate}\n")?;
            }
            Tag::List(None) => {
                self.write("\\end{itemize}\n")?;
            }
            Tag::Item => {
                self.write_newline()?;
            }
            Tag::Emphasis => {
                self.write("}")?;
            }
            Tag::Strong => {
                self.write("}")?;
            }
            Tag::Strikethrough => {
                self.write("}")?;
            }
            Tag::Link(_, _, _) => {
                self.write("}")?;
            }
            Tag::Image(_, _, _) => (), // shouldn't happen, handled in start
            Tag::FootnoteDefinition(_) => {
                self.write("}\n")?;
            }
        }
        Ok(())
    }

    // run raw text, consuming end tag
    fn consume_text(&mut self) -> io::Result<()> {
        let mut nest = 0;
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(_) => nest += 1,
                Event::End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                }
                Event::Html(_) | Event::Code(_) | Event::Text(_) => {}
                Event::SoftBreak | Event::HardBreak | Event::Rule => {}
                Event::FootnoteReference(name) => {
                    let len = self.numbers.len() + 1;
                    let _r = *self.numbers.entry(name).or_insert(len);
                }
                Event::TaskListMarker(true) => {}
                Event::TaskListMarker(false) => {}
            }
        }
        Ok(())
    }
}
