use std::{env, fs, io::{stdin, stdout, Write}, path::PathBuf, process::exit};

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
 
fn resolve_path(path: &str, current_dir: &PathBuf) -> String {
    let mut resolved_path = PathBuf::new();
    
    // tilde support
    if path.starts_with("~") {
        if let Some(home_dir) = home::home_dir() {
            resolved_path.push(home_dir);
            if path.len() > 1 { // means we have ~/something
                resolved_path.push(&path[2..]);
            }
        } else { // cant find home dir for some reason so i go root
            resolved_path.push("/");
            resolved_path.push(if path.len() > 1 { &path[2..] } else { "" });
        }
    } else if path.starts_with("/") { // absolute path
        resolved_path.push(path);
    } else { // relative path
        resolved_path = current_dir.clone();
        resolved_path.push(path);
    }
    
    // parse . and ..
    if let Ok(canonical) = fs::canonicalize(&resolved_path) {
        canonical.display().to_string()
    } else {
        resolved_path.display().to_string()
    }
}

fn ls(path: &str, current_dir: &PathBuf) -> std::io::Result<()> {
    let resolved_path = resolve_path(path, current_dir);    

    let entries = fs::read_dir(&resolved_path)?;
    for entry in entries {
        let entry = entry?;
        let path_buf = entry.path();

        // take out prefixes
        let display_path = path_buf.display().to_string();
        let display_path = display_path.trim_start_matches(&resolved_path).trim_start_matches("/");

        print!("{} ", display_path);
    }
    println!("\n");
    Ok(())
}

fn pwd(current_dir: &PathBuf) {
    println!("{}", current_dir.display());
}

fn cd(path: &str, current_dir: &mut PathBuf) {
    let resolved_path = resolve_path(path, current_dir);
    let new_path = PathBuf::from(resolved_path);
    if new_path.is_dir() {
        *current_dir = new_path;
    } else {
        println!("{} is not a directory", new_path.display());
    }
}

fn echo(args: Vec<&str>) {
    for arg in args {
        print!("{} ", arg);
    }
    println!("\n");
}

fn cat(path: &str, current_dir: &PathBuf) {
    let resolved_path = resolve_path(path, current_dir);
    let path_buf = PathBuf::from(resolved_path);

    if !path_buf.is_file() {
        println!("{} is not a file", path_buf.display());
        return;
    }

    fs::read_to_string(&path_buf).map(|content| {
        println!("{}", content);
    }).unwrap();
}

fn handle_command(command: &str, current_dir: &mut PathBuf) {
    let mut parts = command.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();

    match cmd {
        "exit" => exit(0),
        "help" => help(),
        "ls" => {
            let path = args.get(0).unwrap_or(&".");
            ls(path, current_dir).unwrap();
        },
        "pwd" => {
            pwd(current_dir);
        },
        "cd" => {
            let path = args.get(0).unwrap_or(&".");
            cd(path, current_dir);
        },
        "echo" => {
            echo(args);
        },
        "cat" => {
            let path = args.get(0).unwrap();
            cat(path, current_dir);
        },
        _ => println!("Unknown command: {}", cmd),
    }
}

fn main() {    
    let mut current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

    loop {
        print!("{}", PROMPT);
        stdout().flush().unwrap(); // idc
        let mut user_input = String::new();
        let _ = stdin().read_line(&mut user_input);

        let command = user_input.trim();
        handle_command(command, &mut current_dir);
    }
}
