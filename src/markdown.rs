use maud::{Markup, PreEscaped, Render};
use pulldown_cmark::{html, Options, Parser};

/// Renders a block of Markdown using `pulldown-cmark`.
pub struct Markdown<T: AsRef<str>>(pub T);

impl<T: AsRef<str>> Render for Markdown<T> {
    fn render(&self) -> Markup {
        // Generate raw HTML
        let mut unsafe_html = String::new();
        let parser = Parser::new_ext(self.0.as_ref(), Options::all());
        html::push_html(&mut unsafe_html, parser);
        // Sanitize it with ammonia
        let safe_html = ammonia::clean(&unsafe_html);
        PreEscaped(safe_html)
    }
}
