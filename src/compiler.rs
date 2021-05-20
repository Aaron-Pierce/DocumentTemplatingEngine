use std::{env::Args, fs::read_to_string, path::Path};

use fs_extra::dir::CopyOptions;

pub struct Compiler {
    article_directory: String,
    template_directory: String,
}

impl Compiler {
    pub fn get_article_directory(&self) -> String {
        self.article_directory.clone()
    }

    pub fn new(article_directory: &str, template_directory: &str) -> Compiler {
        Compiler {
            //we have to repeat the field name to call .to_string() against them
            //instead of just writing
            //  {
            //      article_directory,
            //      template_directory
            //  }
            // i.e we can't use the Field Init Shorthand.
            article_directory: article_directory.to_string(),
            template_directory: template_directory.to_string(),
        }
    }

    // Parses out commands and runs the corresponding methods
    pub fn interpret_arguments(&self, args: &mut Args) {
        if args.len() > 0 {
            let command = args.next().unwrap().to_lowercase();
            if command == "compile" {
                args.for_each(|e| {
                    if e == "all" {
                        self.compile_all();
                    } else {
                        self.compile(&e);
                    }
                });
            } else if command == "publish" {
                self.compile_all();
                self.publish();
            }
        }
    }

    pub fn compile_all(&self) {
        std::fs::read_dir(self.get_article_directory())
            .unwrap()
            .for_each(|file_result| {
                if file_result.is_ok() {
                    let file_name = file_result.unwrap().file_name();
                    match file_name.to_string_lossy().strip_suffix(".html") {
                        Some(filename) => self.compile(&filename.to_string()),
                        None => {}
                    }
                }
            });

        self.copy_static_files();
        println!("Compiled all templates.")
    }

    pub fn compile(&self, name: &str) {
        use std::fs::File;
        use std::io::Write;

        println!("Requested compile of {:?}", name);

        let built_article = self.build_article(name);
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

    //copies the static files from public/ to the built directory
    pub fn copy_static_files(&self) {
        let copy_options = CopyOptions {
            overwrite: true,
            skip_exist: false,
            content_only: true,
            ..CopyOptions::new()
        };

        println!("Copying static directory contents to build directory...");

        fs_extra::dir::copy("./src/public", "./built", &copy_options)
            .expect("Could not copy contents of ./src/public");
    }

    //this is pretty weak. We need a better publish
    //this just runs the following:
    // git add built/*
    // git commit -m "TODO: add a commit message"
    // git push
    // and relies on the local git config to handle push correctly
    fn publish(&self) {
        std::process::Command::new("git")
            .args(&["add", "."])
            .current_dir("./built")
            .status()
            .expect("Failed to add all files");

        std::process::Command::new("git")
            .args(&["commit", "-m", "TODO: add a real commit message"])
            .current_dir("./built")
            .status()
            .expect("Failed to commit");

        std::process::Command::new("git")
            .args(&["push"])
            .current_dir("./built")
            .status()
            .expect("Failed to push");

        println!("Published")
    }

    // This is where you programatically define bangs.
    // I generally use some strange character sequence to identify places where
    // you want to fill in bang content
    fn expand_bang(&self, bang_name: &str, args: Vec<String>) -> String {
        match bang_name {
            "makeTitle" => {
                let mut template_data =
                    read_to_string(format!("{}{}", self.template_directory, "title.html")).unwrap();

                template_data = template_data.replace(
                    "*+*",
                    args.get(0).unwrap_or(&String::from("Error: Title Missing")),
                );
                template_data =
                    template_data.replace("?*?", args.get(1).unwrap_or(&String::from("")));

                return template_data;
            }
            "archiveEntry" => {
                let mut template_data = read_to_string(format!(
                    "{}{}",
                    self.template_directory, "archiveEntry.html"
                ))
                .unwrap();

                template_data =
                    template_data.replace("?+?", args.get(0).unwrap_or(&String::from("")));

                template_data =
                    template_data.replace("*+*", args.get(1).unwrap_or(&String::from("")));

                template_data =
                    template_data.replace("?d", args.get(2).unwrap_or(&String::from("")));
                return template_data;
            }
            "makeMeta" => {
                let mut template_data =
                    read_to_string(format!("{}{}", self.template_directory, "meta.html")).unwrap();

                while (template_data.contains("param:")) {
                    template_data = template_data
                        .replace("param:title", args.get(0).unwrap_or(&String::from("")));
                    template_data =
                        template_data.replace("param:description", args.get(1).unwrap_or(&String::from("")));
                }
                return template_data;
            }
            _ => String::new(),
        }
    }

    fn build(&self, template_location: &str, reverse_template_content: Option<&str>) -> String {
        println!("Building template... {:#?}", template_location);
        let template_data_result = read_to_string(template_location);

        let mut template_data = match template_data_result {
            Ok(data) => data,
            Err(err) => {
                eprintln!("{:?}", err);
                String::new()
            }
        };

        if template_data.len() == 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            template_data = read_to_string(template_location).unwrap();
        }

        let first_line = template_data.lines().next().unwrap();

        let reverse_template: Option<String>;
        if first_line.starts_with("*") {
            reverse_template = Some(first_line.strip_prefix("*").unwrap().to_string());
            // println!(
            //     "{} wants to be wrapped in {}",
            //     template_location,
            //     reverse_template.clone().unwrap()
            // );
            template_data = template_data.replace(first_line, "");
        } else {
            reverse_template = None;
        }

        // while the template has un-parsed bangs
        while template_data.contains("!{") {
            let parsed = self.parse_command(&template_data, "!{*}");

            template_data.replace_range(
                parsed.start_index..(parsed.start_index + parsed.total_length),
                &self.expand_bang(&parsed.name[..], parsed.arguments)[..],
            );
        }

        // while the template has un-parsed commands
        while template_data.contains("${") {
            let parsed = self.parse_command(&template_data, "${*}");
            if parsed.name.starts_with("-") && reverse_template_content.is_some() {
                //this is the reverse template
                template_data.replace_range(
                    parsed.start_index..(parsed.start_index + parsed.total_length),
                    reverse_template_content.unwrap(),
                );
            } else {
                template_data.replace_range(
                    parsed.start_index..(parsed.start_index + parsed.total_length),
                    &self.build_template(&parsed.name, None)[..],
                )
            }
        }

        match reverse_template {
            Some(string) => self.build_template(&string, Some(&template_data)),
            None => template_data,
        }
    }

    //specific methods to call the generic build on.
    //articles have no reverse template content,
    //meaning other templates cannot ask to be wrapped in
    //an article, because that seems strange
    fn build_template(
        &self,
        template_name: &str,
        reverse_template_content: Option<&str>,
    ) -> String {
        self.build(
            &format!("{}{}{}", self.template_directory, template_name, ".html"),
            reverse_template_content,
        )
    }

    // only necessary so that you don't have to include a None argument
    // every time you call build for an article.
    fn build_article(&self, article_name: &str) -> String {
        self.build(
            &format!("{}{}{}", self.article_directory, article_name, ".html"),
            None,
        )
    }

    // finds an instance of a command somewhere in the data parameter
    // returns a TemplateCommand with the arguments that the command
    // have and the char position of the command so you can replace
    // the text.
    // It takes a command format that expects an asterisk where
    // the command name is.
    // For example, to find a bang, you would use !{*},
    // which matches anything that starts with "!{", has the name here, and then ends with "}".
    // so the command_format "!{*}" will match "!{myBang}" or "!{makeTitle}" or "!{todaysDate}"
    // then it will search the end of that command for an argument in square brackets,
    // with more arguments following the brackets
    // for example, "${myCommand}[myArgument1]" or "#{myCommand}[myArgument1][myArgument2][myArgument3]"
    // however, arguments are hardcoded and the same between all command_formats
    // therefore you don't include them in command_format.
    // a few possible command formats are: "${*}", "#{*}", "~{*}"
    // it is critically important that * is not the last character
    // of a command_format, because it will match all of the rest
    // of the template as a command_name, and then it will look for
    // brackets after the end of the file, which will not end well.
    // so all commands need to have their * enclosed in some way.
    fn parse_command(&self, data: &str, command_format: &str) -> TemplateCommand {
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

        // println!("Parsed out command {}", command_name);

        let mut arguments: Vec<String> = Vec::new();
        let mut argument_size_in_text: usize = 0;

        // each loop advances the char iterator all the way through a command
        // so this condition may seem like it only matches continuous
        // repetitions of an open bracket ("[[[", or something) but because the
        // loop body advances char_iterator, it will loop twice on the following
        // string: "[arg1][arg2]" because the loop body will advance the char
        // iterator 6 characters (the length of "[arg1]"), in the process of
        // parsing out its value.
        // note that it doesn't skip whitespace, so a command like "!{command1}[argument1] [argument2]"
        // will only have argument1 parsed out and "[argument2]" will be treated as text
        // that has nothing to do with the command
        while char_iterator.next().unwrap_or('_') == '[' {
            // println!("Bang also has arguments!");
            let mut argument = String::new();
            while char_iterator.peek().is_some() {
                let next = char_iterator.next();
                if next.unwrap_or(']') == ']' {
                    break;
                };
                argument.push(next.unwrap())
            }
            // println!("Which is {:?}", argument);

            // this 2 compensates for the two brackets that
            // surround each argument
            // we have the length of the arguments plus
            // the two bracket characters we omitted
            argument_size_in_text += 2 + argument.len();
            arguments.push(argument);
        }

        return TemplateCommand {
            arguments: arguments,
            start_index: index,
            total_length: command_prefix.len()
                + command_name.len()
                + command_suffix.len()
                + argument_size_in_text,
            name: command_name,
        };
    }
}

// a struct that carries the information about a TemplateCommand
// for example, if the command is "!{command_name}[arg1][arg2]"
// the TemplateCommand struct should be
// {
//   name: command_name,
//   arguments: Vec["arg1", "arg2"]
//   start_index: 0,
//   total_length: 27
// }
// So you know you're invoking the command_name command
// with 2 arguments of value arg1 and arg2.
// within the greater body of text, the command is at index 0
// and the command is 27 characters long, so that you know
// how many characters you need to replace
// with whatever you want your command to evaluate to
struct TemplateCommand {
    name: String,
    arguments: Vec<String>,
    start_index: usize,
    total_length: usize,
}
