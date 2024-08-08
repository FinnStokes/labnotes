use std::convert::TryFrom;
use std::path::PathBuf;

use clap::Parser;

use labnotes::{Note, NoteID};

#[derive(Debug, Parser)]
#[command(
    name = "lab2tex",
    about = "Converts a markdown file into a latex file."
)]
struct Args {
    /// Markdown file to convert
    input: PathBuf,
}

fn main() {
    let args = Args::parse();
    match Note::load(NoteID::try_from("index").unwrap(), args.input) {
        Ok(note) => {
            let tex = note.render_tex();
            print!("{}", tex);
        }
        Err(_) => {
            eprintln!("Error loading note!");
        }
    };
}
