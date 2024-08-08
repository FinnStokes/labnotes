use maud::{Markup, PreEscaped, Render};
use pulldown_cmark::{
    html, Alignment, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag, TagEnd,
};
use pulldown_cmark_escape::{escape_href, escape_html};
use regex::Regex;
use std::collections::HashMap;

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

struct KatexMiddleware;

impl KatexMiddleware {
    fn new() -> KatexMiddleware {
        KatexMiddleware
    }

    fn map<'a>(&'_ mut self, event: Event<'a>) -> Option<Event<'a>> {
        match event {
            Event::DisplayMath(text) => {
                let opts = katex::Opts::builder().display_mode(true).build().unwrap();
                Some(Event::Html(CowStr::from(
                    katex::render_with_opts(text.as_ref(), opts).unwrap_or_else(|e| match e {
                        katex::Error::JsExecError(s) => {
                            format!("<div class=\"todo\">{}</div>", s)
                        }
                        _ => panic!("{}", e),
                    }),
                )))
            }
            Event::InlineMath(text) => Some(Event::Html(CowStr::from(
                katex::render(text.as_ref()).unwrap_or_else(|e| match e {
                    katex::Error::JsExecError(s) => {
                        format!("<span class=\"todo\">{}</span>", s)
                    }
                    _ => panic!("{}", e),
                }),
            ))),
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
    W: pulldown_cmark_escape::StrWrite,
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
    fn write_newline(&mut self) -> Result<(), W::Error> {
        self.end_newline = true;
        self.writer.write_str("\n")
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    #[inline]
    fn write(&mut self, s: &str) -> Result<(), W::Error> {
        self.writer.write_str(s)?;

        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    pub fn run(mut self) -> Result<(), W::Error> {
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
                    self.write(r"\mintinline{text}{")?;
                    self.write(&text)?;
                    self.write(r"}")?;
                }
                Event::InlineMath(text) => {
                    self.write(r"$")?;
                    self.write(&text)?;
                    self.write(r"$")?;
                }
                Event::DisplayMath(text) => {
                    self.write("\\begin{align*}")?;
                    self.write(&text)?;
                    self.write("\\end{align*}")?;
                }
                Event::Html(html) => {
                    self.write("\\begin{verbatim}")?;
                    self.write(&html)?;
                    self.write("\\end{verbatim}")?;
                }
                Event::InlineHtml(text) => {
                    self.write(r"\mintinline{text}{")?;
                    self.write(&text)?;
                    self.write(r"}")?;
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
    fn start_tag(&mut self, tag: Tag<'a>) -> Result<(), W::Error> {
        match tag {
            Tag::Paragraph => {
                if self.end_newline {
                    self.write("\n")
                } else {
                    self.write("\n\n")
                }
            }
            Tag::Heading { level, .. } => {
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
                    write!(&mut self.writer, "\n\\{}{{", section)
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
            Tag::BlockQuote(_) => {
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
            Tag::Link {
                link_type: LinkType::Email,
                dest_url: dest,
                ..
            } => {
                self.write("\\href{mailto:")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("}{")
            }
            Tag::Link { dest_url: dest, .. } => {
                self.write("\\href{")?;
                escape_href(&mut self.writer, &dest)?;
                self.write("}{")
            }
            Tag::Image { dest_url: dest, .. } => {
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
            Tag::HtmlBlock => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                self.write("\\begin{verbatim}\n")
            }
            Tag::MetadataBlock(_) => {
                if !self.end_newline {
                    self.write_newline()?;
                }
                self.write("\\begin{comment}\n")
            }
        }
    }

    fn end_tag(&mut self, tag: TagEnd) -> Result<(), W::Error> {
        match tag {
            TagEnd::Paragraph => {}
            TagEnd::Heading { .. } => {
                self.write("}\n")?;
            }
            TagEnd::Table => {
                self.write("\\end{tabular}\n\\end{center}\n")?;
            }
            TagEnd::TableHead => {
                self.write("\\hline\n")?;
            }
            TagEnd::TableRow => {}
            TagEnd::TableCell => {
                self.table_cell_index += 1;
                if self.table_cell_index == self.table_cells {
                    self.write("\\\\\n")?;
                } else {
                    self.write(" & ")?;
                }
            }
            TagEnd::BlockQuote => {
                self.write("\\end{quotation}\n")?;
            }
            TagEnd::CodeBlock => {
                self.write("\\end{minted}\n")?;
            }
            TagEnd::List(true) => {
                self.write("\\end{enumerate}\n")?;
            }
            TagEnd::List(false) => {
                self.write("\\end{itemize}\n")?;
            }
            TagEnd::Item => {
                self.write_newline()?;
            }
            TagEnd::Emphasis => {
                self.write("}")?;
            }
            TagEnd::Strong => {
                self.write("}")?;
            }
            TagEnd::Strikethrough => {
                self.write("}")?;
            }
            TagEnd::Link { .. } => {
                self.write("}")?;
            }
            TagEnd::Image { .. } => (), // shouldn't happen, handled in start
            TagEnd::FootnoteDefinition => {
                self.write("}\n")?;
            }
            TagEnd::HtmlBlock => {
                self.write("\\end{verbatim}\n")?;
            }
            TagEnd::MetadataBlock(_) => {
                self.write("\\end{comment}\n")?;
            }
        }
        Ok(())
    }

    // run raw text, consuming end tag
    fn consume_text(&mut self) -> Result<(), W::Error> {
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
                Event::Html(_)
                | Event::InlineHtml(_)
                | Event::DisplayMath(_)
                | Event::InlineMath(_)
                | Event::Code(_)
                | Event::Text(_) => {}
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
