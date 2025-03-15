use std::{env, fs, io::{stdin, stdout, Write}, os::unix::fs::PermissionsExt, path::PathBuf, process::{exit, Command, Stdio}};

const PROMPT: &str = "kebda $ ";

const COMMAND_LIST: &[&str] = &[
    "help",
    "exit",
    "ls",
    "cd",
    "pwd",
    "echo",
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

fn ls(path: &str, current_dir: &PathBuf){
    let resolved_path = resolve_path(path, current_dir);    

    let entries = match fs::read_dir(&resolved_path) {
        Ok(entries) => entries,
        Err(e) => {
            println!("Failed to read directory: {}", e);
            return;
        }
    };
    
    for entry in entries {
        let entry = entry.unwrap();
        let path_buf = entry.path();

        // take out prefixes
        let display_path = path_buf.display().to_string();
        let display_path = display_path.trim_start_matches(&resolved_path).trim_start_matches("/");

        print!("{} ", display_path);
    }
    println!("\n");
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

fn find_executable(cmd: &str) -> Option<PathBuf> {
    if let Ok(paths) = env::var("PATH") {
        for path in paths.split(':') {
            let mut cmd_path = PathBuf::from(path);
            cmd_path.push(cmd);
            if cmd_path.exists() {
                if let Ok(metadata) = fs::metadata(&cmd_path) {
                    let permissions = metadata.permissions();
                    if permissions.mode() & 0o111 != 0 {
                        return Some(cmd_path);
                    }
                }
            }
        }
    }
    None
}

fn parse_command(command: &str) -> (String, Vec<String>) {
    let mut parts = command.trim().split_whitespace();
    let cmd = parts.next().unwrap_or("").to_string();
    let args: Vec<String> = parts.map(|s| s.to_string()).collect();
    (cmd, args)
}

fn run_builtin(cmd: &str, args: &[String], current_dir: &mut PathBuf) -> Result<(), String> {
    match cmd {
        "exit" => {
            exit(0);
        },
        "help" => {
            help();
            Ok(())
        },
        "ls" => {
            let path = args.get(0).map(|s| s.as_str()).unwrap_or(".");
            ls(path, current_dir);
            Ok(())
        },
        "pwd" => {
            pwd(current_dir);
            Ok(())
        },
        "cd" => {
            let path = args.get(0).map(|s| s.as_str()).unwrap_or(".");
            cd(path, current_dir);
            Ok(())
        },
        "echo" => {
            let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            echo(str_args);
            Ok(())
        },
        _ => Err("Unknown command".to_string()),
    }
}

fn run_pipeline(commands: Vec<&str>, current_dir: &mut PathBuf) {
    if commands.is_empty() {
        return;
    }

    if commands.len() == 1 { // only 1 cmd
        let (cmd, args) = parse_command(commands[0]);
        
        // check builtins
        if run_builtin(&cmd, &args, current_dir).is_ok() {
            return;
        }
        
        // check external
        if let Some(cmd_path) = find_executable(&cmd) {
            let _ = Command::new(cmd_path)
                .args(args)
                .current_dir(current_dir)
                .status();
            return;
        } else {
            println!("Unknown command: {}", cmd);
            return; 
        }
    }

    // pipeline
    let mut prev_out = None;
    let mut procs = Vec::new();
    
    for (i, cmd_str) in commands.iter().enumerate() {
        let (cmd, args) = parse_command(cmd_str);
        let last_cmd = i == commands.len() - 1;
        
        // find executable
        let cmd_path = match find_executable(&cmd) {
            Some(path) => path,
            None => {
                println!("Unknown command: {}", cmd);
                return;
            }
        };
        
        let mut command = Command::new(cmd_path);
        command.args(args).current_dir(&current_dir);
        
        if let Some(stdout) = prev_out.take() {
            command.stdin(stdout);
        }
        
        if !last_cmd {
            command.stdout(Stdio::piped());
        }
        
        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(e) => {
                println!("Failed to spawn command: {}", e);
                return;
            }
        };
        
        if !last_cmd {
            prev_out = child.stdout.take();
        }
        
        procs.push(child);
    }
    
    for child in procs.iter_mut() {
        let _ = child.wait();
    }
}

fn handle_command(command: &str, current_dir: &mut PathBuf) {
    let commands: Vec<&str> = command.split('|').collect();
    run_pipeline(commands, current_dir);
}

fn main() {    
    let mut current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

    loop {
        print!("{}", PROMPT);
        stdout().flush().unwrap(); // idc
        let mut user_input = String::new();
        let _ = stdin().read_line(&mut user_input);

        let command = user_input.trim();
        if !command.is_empty() {
            handle_command(command, &mut current_dir);
        }
    }
}
