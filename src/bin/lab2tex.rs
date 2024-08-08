#![feature(proc_macro_hygiene, decl_macro)]

use std::convert::TryFrom;
use std::path::PathBuf;

use structopt::StructOpt;

use labnotes::{Note, NoteID};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "lab2tex",
    about = "Converts a markdown file into a latex file."
)]
struct Args {
    /// Markdown file to convert
    input: PathBuf,
}

#[paw::main]
fn main(args: Args) {
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
