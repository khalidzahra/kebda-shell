use std::{io::{stdin, stdout, Write}, process::exit, fs};

const PROMPT: &str = "kebda $ ";

const COMMAND_LIST: &[&str] = &[
    "help",
    "exit",
    "ls",
    "cd",
    "pwd",
    "echo",
    "cat",
];

fn help() {
    println!("Available commands: {}", COMMAND_LIST.join(", "));
}

fn ls(path: &str) -> std::io::Result<()> {
    // tilde support
    let new_path = if path.contains("~") {
        let home = match home::home_dir() {
            Some(path) => path.display().to_string(),
            _ => String::from("/"),
        };
        path.replace("~", &home)
    } else {
        path.to_string()
    };

    let entries = fs::read_dir(&new_path)?;
    for entry in entries {
        let entry = entry?;
        let path_buf = entry.path();

        // take out prefixes
        let display_path = path_buf.display().to_string();
        let display_path = display_path.trim_start_matches(&new_path).trim_start_matches("/");

        print!("{} ", display_path);
    }
    println!("\n");
    Ok(())
}

fn handle_command(command: &str) {
    let mut parts = command.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    match cmd {
        "exit" => exit(0),
        "help" => help(),
        "ls" => {
            let path = args.get(0).unwrap_or(&".");
            ls(path).unwrap();
        },
        _ => println!("Unknown command: {}", cmd),
    }
}

fn main() {
    loop {
        print!("{}", PROMPT);
        stdout().flush().unwrap(); // idc
        let mut user_input = String::new();
        let _ = stdin().read_line(&mut user_input);

        let command = user_input.trim();
        handle_command(command);
    }
}
