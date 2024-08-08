#![feature(proc_macro_hygiene, decl_macro)]

use std::path::PathBuf;

use rocket::{get, routes, State};
use structopt::StructOpt;

use labnotes::{LabBook, Note, NoteID, Theme};

#[get("/", format = "html")]
fn index(book: State<LabBook>) -> Option<Note<'static>> {
    book.index().ok()
}

#[get("/<id>", format = "html")]
fn note<'a>(id: NoteID<'a>, book: State<LabBook>) -> Option<Note<'a>> {
    book.note(id).ok()
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "labnotes",
    about = "Serves a directory of markdown files as a simple website."
)]
struct Args {
    /// Directory that contains the markdown files
    #[structopt(default_value = ".")]
    dir: PathBuf,

    /// Use light theme instead of dark theme
    #[structopt(long)]
    light: bool,
}

#[paw::main]
fn main(args: Args) {
    rocket::ignite()
        .mount("/", routes![index, note])
        .manage(LabBook::new(args.dir))
        .manage(Theme::new(args.light))
        .launch();
}
