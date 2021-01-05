#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::{fs::read_to_string, path::Path};

const TEMPLATE_LOCATION: &str = "./src/templates/";
const ARTICLE_LOCATION: &str = "./src/articles/";

fn expand_bang(bang_name: &str, args: Vec<String>) -> String {
    match bang_name {
        "makeTitle" => {
            let mut template_data =
                read_to_string(format!("{}{}", TEMPLATE_LOCATION, "title.html")).unwrap();

            template_data = template_data.replace(
                "*+*",
                args.get(0).unwrap_or(&String::from("Error: Title Missing")),
            );
            template_data = template_data.replace("?*?", args.get(1).unwrap_or(&String::from("")));

            return template_data;
        }
        "archiveEntry" => {
            let mut template_data =
                read_to_string(format!("{}{}", TEMPLATE_LOCATION, "archiveEntry.html")).unwrap();

            template_data = template_data.replace("?+?", args.get(0).unwrap_or(&String::from("")));

            template_data = template_data.replace("*+*", args.get(1).unwrap_or(&String::from("")));

            template_data = template_data.replace("?d", args.get(2).unwrap_or(&String::from("")));
            return template_data;
        }
        _ => String::new(),
    }
}

struct Command {
    name: String,
    arguments: Vec<String>,
    start_index: usize,
    total_length: usize,
}

fn parse_command(data: &String, command_format: &str) -> Command {
    let mut split_command = command_format.split("*");
    let command_prefix = split_command.next().unwrap();
    let command_suffix = split_command.next().unwrap();

    let index = data.find(command_prefix).unwrap();

    let mut char_iterator = data.chars().peekable();
    char_iterator.nth(index + (command_prefix.len() - 1));

    let mut command_name = String::new();
    while char_iterator.peek().is_some() {
        let next = char_iterator.next();
        if next == command_suffix.chars().next() {
            break;
        };
        command_name.push(next.unwrap())
    }

    println!("Parsed out command {}", command_name);

    let mut arguments: Vec<String> = Vec::new();
    let mut argument_size_in_text: usize = 0;

    while char_iterator.next().unwrap_or('_') == '[' {
        println!("Bang also has arguments!");
        let mut argument = String::new();
        while char_iterator.peek().is_some() {
            let next = char_iterator.next();
            if next.unwrap_or(']') == ']' {
                break;
            };
            argument.push(next.unwrap())
        }
        println!("Which is {:?}", argument);
        argument_size_in_text += 2 + argument.len();
        arguments.push(argument);
    }

    return Command {
        arguments: arguments,
        start_index: index,
        total_length: command_prefix.len()
            + command_name.len()
            + command_suffix.len()
            + argument_size_in_text,
        name: command_name,
    };
}

fn build_template(template_name: &String, reverse_template_content: Option<&String>) -> String {
    println!("Built template request: {:#?}", template_name);
    let mut template_data =
        read_to_string(format!("{}{}{}", TEMPLATE_LOCATION, template_name, ".html")).unwrap();

    let first_line = template_data.lines().next().unwrap();

    let reverse_template: Option<String>;
    if first_line.starts_with("*") {
        reverse_template = Some(first_line.strip_prefix("*").unwrap().to_string());
        println!(
            "{} wants to be wrapped in {}",
            template_name,
            reverse_template.clone().unwrap()
        );
        template_data = template_data.replace(first_line, "");
    } else {
        reverse_template = None;
    }

    while template_data.contains("${") {
        let parsed = parse_command(&template_data, "${*}");
        if parsed.name.starts_with("-") && reverse_template_content.is_some() {
            //this is the reverse template
            template_data.replace_range(
                parsed.start_index..(parsed.start_index + parsed.total_length),
                reverse_template_content.unwrap(),
            );
            return template_data;
        }
        template_data.replace_range(
            parsed.start_index..(parsed.start_index + parsed.total_length),
            &build_template(&parsed.name, None)[..],
        )
    }

    match reverse_template {
        Some(string) => build_template(&string, Some(&template_data)),
        None => template_data,
    }
}

fn build_article(article_name: &String) -> String {
    println!("Desires to build article {:#?}", article_name);
    let mut article_data =
        read_to_string(format!("{}{}{}", ARTICLE_LOCATION, article_name, ".html")).unwrap();

    let first_line = article_data.lines().next().unwrap();

    let reverse_template: Option<String>;
    if first_line.starts_with("*") {
        reverse_template = Some(first_line.strip_prefix("*").unwrap().to_string());
        println!(
            "{} wants to be wrapped in {}",
            article_name,
            reverse_template.clone().unwrap()
        );
        article_data = article_data.replace(first_line, "");
    } else {
        reverse_template = None;
    }

    while article_data.contains("!{") {
        let parsed = parse_command(&article_data, "!{*}");

        article_data.replace_range(
            parsed.start_index..(parsed.start_index + parsed.total_length),
            &expand_bang(&parsed.name[..], parsed.arguments)[..],
        );
    }

    while article_data.contains("${") {
        let parsed = parse_command(&article_data, "${*}");
        article_data.replace_range(
            parsed.start_index..(parsed.start_index + parsed.total_length),
            &build_template(&parsed.name, None)[..],
        );
    }

    match reverse_template {
        Some(string) => build_template(&string, Some(&article_data)),
        None => article_data,
    }
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(build_article(&String::from("test")))
}

fn compile(name: &String) {
    use std::fs::File;
    use std::io::Write;

    println!("Requested compile of {:?}", name);

    let built_article = build_article(name);
    let build_path = Path::new("./built/");
    match std::fs::create_dir(build_path) {
        Ok(_) => (),
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::AlreadyExists => {}
                _ => {
                    //induce a panic
                    panic!(err);
                }
            }
        }
    }
    let destination_path = build_path.join(format!("{}{}", name, ".html"));
    let mut file = File::create(destination_path)
        .expect("Could not touch file. Do you have write permission?");
    file.write_all(built_article.as_bytes())
        .expect("Could not write file. Do you have write permissions?");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut args = std::env::args();
    args.next();
    args.for_each(|e| {
        if e == "all" {
            std::fs::read_dir(ARTICLE_LOCATION)
                .unwrap()
                .for_each(|file_result| {
                    if file_result.is_ok() {
                        let file_name = file_result.unwrap().file_name();
                        match file_name.to_string_lossy().strip_suffix(".html") {
                            Some(filename) => compile(&filename.to_string()),
                            None => {}
                        }
                    }
                })
        } else {
            compile(&e);
        }
    });

    let result = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(manual_hello))
            .service(fs::Files::new("/public", "./src/public/"))
            .service(fs::Files::new("/built", "./built"))
    })
    .bind("192.168.1.196:8080")?
    .bind("127.0.0.1:8080")?
    .run()
    .await;

    println!("After server");

    return result;
}
