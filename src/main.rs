#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

// use actix_files as fs;
use actix_web::{App, HttpServer};
use fs_extra::{self, dir::CopyOptions};
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;
mod compiler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use compiler::Compiler;

    let compiler = Compiler {
        article_directory: "src/articles/",
        template_directory: "./src/templates/",
    };

    let mut args = std::env::args();
    args.next();
    args.for_each(|e| {
        if e == "all" {
            std::fs::read_dir(compiler.article_directory)
                .unwrap()
                .for_each(|file_result| {
                    if file_result.is_ok() {
                        let file_name = file_result.unwrap().file_name();
                        match file_name.to_string_lossy().strip_suffix(".html") {
                            Some(filename) => compiler.compile(&filename.to_string()),
                            None => {}
                        }
                    }
                })
        } else {
            compiler.compile(&e);
        }
    });

    let copy_options = CopyOptions {
        overwrite: true,
        skip_exist: false,
        content_only: true,
        ..CopyOptions::new()
    };
    fs_extra::dir::copy("./src/public", "./built", &copy_options)
        .expect("Could not copy contents of ./src/public");

    let result = HttpServer::new(|| {
        App::new()
            .service(actix_files::Files::new("/public", "./src/public/"))
            .service(actix_files::Files::new("/", "./built"))
    })
    .bind("192.168.1.244:8080")?
    .bind("127.0.0.1:8080")?
    .run();

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch("src/articles/", RecursiveMode::Recursive)
        .unwrap();

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    notify::DebouncedEvent::NoticeWrite(pathbuf) => {
                        let article_name = pathbuf.file_name().unwrap().to_str().unwrap();
                        let article_name = String::from(article_name);
                        let article_name = article_name.strip_suffix(".html");
                        match article_name {
                            Some(name) => {
                                println!("Detected change for {:?}, compiling", name);
                                compiler.compile(&String::from(name));
                            },
                            None => eprintln!("Changed article did not end in .html")
                        }
                    },
                    _ => {}
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    return result.await;
}
