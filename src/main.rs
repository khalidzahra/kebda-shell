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

fn handle_command(command: &str) {
    let mut parts = command.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    match cmd {
        "exit" => exit(0),
        "help" => help(),
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
