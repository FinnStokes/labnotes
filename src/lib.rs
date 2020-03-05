#![feature(proc_macro_hygiene, str_strip)]

use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};
use std::fs::read_to_string;
use std::path::PathBuf;

use maud::{html, Markup, DOCTYPE};
use rocket::http::RawStr;
use rocket::request::{FromParam, Request};
use rocket::response::{self, Responder};

mod markdown;

pub use markdown::Markdown;

#[derive(Debug)]
pub enum Error {
    /// NoteID has wrong number of characters
    /// (should be between 1 and 128, inclusive)
    InvalidLength,

    /// NoteID has invalid character
    /// (allowed characters are `a-z`, `A-Z`, `0-9`, `_`, and `-`)
    InvalidCharacter,

    /// Note with given NoteID not found in LabBook
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

/// A collection of notes, stored as markdown files in a single directory
pub struct LabBook {
    dir: PathBuf,
}

impl LabBook {
    /// Load all notes in a specific directory
    pub fn new(dir: PathBuf) -> LabBook {
        LabBook { dir }
    }

    /// Get index page from `{dir}/index.md`
    pub fn index(&self) -> Result<Note<'static>> {
        self.note(NoteID::try_from("index").unwrap())
    }

    /// Get page with specific id from `{dir}/{id}.md`
    pub fn note<'a>(&self, id: NoteID<'a>) -> Result<Note<'a>> {
        let filename: &str = (&id).into();
        let mut filename = PathBuf::from(filename);
        filename.set_extension("md");
        let path = self.dir.join(filename);
        Note::load(id, path)
    }
}

pub struct NoteMetadata;

/// A lab note, consisting of a header containing metadata and
/// a body contianing a markdown string. Can be rendered as html.
pub struct Note<'a> {
    pub id: NoteID<'a>,
    pub header: NoteMetadata,
    pub body: Markdown<String>,
}

impl Note<'_> {
    /// Load note from a file. Should contain yaml-encoded metadata
    /// followed by markdown body.
    pub fn load(id: NoteID, path: PathBuf) -> Result<Note> {
        let body = Markdown(read_to_string(path).or(Err(Error::NotFound))?);
        Ok(Note {
            id,
            header: NoteMetadata,
            body,
        })
    }

    /// Render the Note to html
    pub fn render_html(&self) -> Markup {
        html! {
            (DOCTYPE)
            head {
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.css" integrity="sha384-zB1R0rpPzHqg7Kpt0Aljp8JPLqbXI3bhnPWROx27a9N0Ll6ZP/+DiW/UqRcLbRjq" crossorigin="anonymous";
            }
            body {
                (self.body)
            }
        }
    }
}

impl<'a> Responder<'static> for Note<'a> {
    /// Respond to rocket requests with html rendering of Note
    fn respond_to(self, r: &Request) -> response::Result<'static> {
        self.render_html().respond_to(r)
    }
}

const ALLOWED_CHARS: &'static [u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
const MAX_SIZE: usize = 128;

/// Identifier for a apecific note. A given identifier corresponds
/// to a markdown file `{id}.md`. Must be between 1 and 128 alphanumeric
/// characters, `-`, or `_`
pub struct NoteID<'a>(Cow<'a, str>);

impl<'a> TryFrom<&'a str> for NoteID<'a> {
    type Error = Error;

    fn try_from(string: &'a str) -> Result<Self> {
        if string.len() > MAX_SIZE || string.len() < 1 {
            Err(Error::InvalidLength)
        } else if !string.bytes().all(|c| ALLOWED_CHARS.contains(&c)) {
            Err(Error::InvalidCharacter)
        } else {
            Ok(NoteID(Cow::Borrowed(string)))
        }
    }
}

impl TryFrom<String> for NoteID<'static> {
    type Error = Error;

    fn try_from(string: String) -> Result<Self> {
        if string.len() > MAX_SIZE || string.len() < 1 {
            Err(Error::InvalidLength)
        } else if !string.bytes().all(|c| ALLOWED_CHARS.contains(&c)) {
            Err(Error::InvalidCharacter)
        } else {
            Ok(NoteID(Cow::Owned(string)))
        }
    }
}

impl Display for NoteID<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Into<String> for &NoteID<'_> {
    fn into(self) -> String {
        String::from(self.0.as_ref())
    }
}

impl<'a> Into<&'a str> for &'a NoteID<'a> {
    fn into(self) -> &'a str {
        &self.0
    }
}

/// Returns an instance of `NoteID` if the path segment is a valid ID.
/// Otherwise returns the invalid ID as the `Err` value.
impl<'a> FromParam<'a> for NoteID<'a> {
    type Error = &'a RawStr;

    fn from_param(param: &'a RawStr) -> std::result::Result<NoteID<'a>, &'a RawStr> {
        match NoteID::try_from(param as &str) {
            Ok(id) => Ok(id),
            Err(_) => Err(param),
        }
    }
}
