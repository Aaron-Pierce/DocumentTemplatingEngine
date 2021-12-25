#![warn(
    clippy::all
)]

// use actix_files as fs;
use actix_web::{App, HttpServer};
use notify::{watcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
mod compiler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let compiler: compiler::Compiler = compiler::Compiler::new("src/articles/", "./src/templates/");

    let mut args = std::env::args();
    // skip the executable name argument
    args.next();

    compiler.interpret_arguments(&mut args);

    let port = "8080";

    HttpServer::new(|| {
        App::new()
            .service(actix_files::Files::new("/public", "./src/public/"))
            .service(actix_files::Files::new("/", "./built"))
    })
    .bind(format!("{}{}", "127.0.0.1:", port))?
    .run();

    println!("Running server on port: {}", port);

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    // Adds the article_directory path to be watched
    watcher
        .watch(compiler.get_article_directory(), RecursiveMode::Recursive)
        .unwrap();

    //wait for files to change in the article directory
    loop {
        match rx.recv() {
            Ok(event) => match event {
                notify::DebouncedEvent::NoticeWrite(pathbuf) => {
                    //format it so that the compiler will take it
                    let article_name = pathbuf.file_name().unwrap().to_str().unwrap();
                    let article_name = String::from(article_name);
                    let article_name = article_name.strip_suffix(".html");
                    match article_name {
                        Some(name) => {
                            println!("Detected change for {:?}, compiling", name);
                            compiler.compile(name);
                        }
                        None => eprintln!("Changed article did not end in .html"),
                    }
                }
                _ => {}
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
