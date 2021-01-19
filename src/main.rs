#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

// use actix_files as fs;
use fs_extra::{self, dir::CopyOptions};
use actix_web::{App, HttpServer};
mod compiler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use compiler::Compiler;
    
    let compiler = Compiler{
        article_directory: "./src/articles/",
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
    fs_extra::dir::copy("./src/public", "./built", &copy_options).expect("Could not copy contents of ./src/public");

    let result = HttpServer::new(|| {
        App::new()
            .service(actix_files::Files::new("/public", "./src/public/"))
            .service(actix_files::Files::new("/", "./built"))
    })
    .bind("192.168.1.244:8080")?
    .bind("127.0.0.1:8080")?
    .run()
    .await;

    println!("After server");

    return result;
}
