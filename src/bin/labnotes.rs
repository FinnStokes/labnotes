use std::path::PathBuf;

use clap::Parser;
use maud::Markup;
use rocket::{fs::FileServer, get, launch, routes, State};

use labnotes::{LabBook, NoteID, Theme};

#[get("/", format = "html")]
fn index(book: &State<LabBook>, theme: &State<Theme>) -> Option<Markup> {
    book.index().ok().map(|note| note.render_html(&theme))
}

#[get("/<id>", format = "html")]
fn note<'a>(id: NoteID<'a>, book: &State<LabBook>, theme: &State<Theme>) -> Option<Markup> {
    book.note(id).ok().map(|note| note.render_html(&theme))
}

#[derive(Parser, Debug)]
#[command(
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

#[launch]
fn rocket() -> _ {
    let args = Args::parse();
    let staticdir = args.dir.join("static");
    let rocket = rocket::build()
        .mount("/", routes![index, note])
        .manage(LabBook::new(args.dir))
        .manage(Theme::new(args.light));

    if staticdir.exists() {
        rocket.mount("/static", FileServer::from(staticdir))
    } else {
        rocket
    }
}
