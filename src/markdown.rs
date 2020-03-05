use maud::{Markup, PreEscaped, Render};
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag};

/// Renders a block of Markdown using `pulldown-cmark`.
pub struct Markdown<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for Markdown<T> {
    fn render(&self) -> Markup {
        // Generate raw HTML
        let parser = Parser::new_ext(self.0.as_ref(), Options::all());

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
                        katex::render_with_opts(text.as_ref(), opts).unwrap(),
                    )))
                } else {
                    Some(Event::Text(text))
                }
            }
            Event::Code(code) => {
                let s = code.as_ref();
                if let Some(text) = s.strip_prefix("$").and_then(|s| s.strip_suffix("$")) {
                    Some(Event::Html(CowStr::from(katex::render(text).unwrap())))
                } else {
                    Some(Event::Code(code))
                }
            }
            e => Some(e),
        }
    }
}
