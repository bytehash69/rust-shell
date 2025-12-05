#![allow(warnings)]
#[allow(unused_imports)]
use std::io::{self, Write};
use std::{
    env::{self},
    fs,
    os::unix::fs::PermissionsExt,
    process::Command,
};

fn main() {
    while true {
        let commands = vec!["echo", "type", "pwd", "cd", "ls", "exit"];
        let current_path = env::current_dir().unwrap();
        let username = std::env::var("USER").unwrap_or("unknown".into());
        let hostname = std::fs::read_to_string("/etc/hostname")
            .unwrap()
            .trim()
            .to_string();

        print!(
            "\x1b[32m{}\x1b[0m@{} \x1b[32m{}\x1b[0m> ",
            username,
            hostname,
            short_path(&current_path)
        );
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if command.contains("\x1b") {
            continue;
        }
        let parts = command.split_whitespace().collect::<Vec<&str>>();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "echo" => println!("{}", parts[1..].join(" ")),
            "type" => {
                if commands.contains(&parts[1]) {
                    println!("{} is a shell builtin", parts[1]);
                } else {
                    match env::var("PATH") {
                        Ok(paths) => {
                            let mut found = false;
                            for path in env::split_paths(&paths) {
                                let path = path.join(parts[1]);
                                if path.exists() {
                                    let metadata =
                                        fs::metadata(&path).expect("Failed to get metadata");
                                    let permissions = metadata.permissions();
                                    let mode = permissions.mode();
                                    if mode & 0o111 != 0 {
                                        println!("{} is {}", parts[1], path.display());
                                        found = true;
                                        break;
                                    }
                                }
                            }
                            if !found {
                                println!("{}: not found", parts[1])
                            }
                        }
                        Err(_) => {
                            println!("{}: not found", parts[1..].join(" "));
                        }
                    }
                }
            }
            "pwd" => println!("{}", env::current_dir().unwrap().display()),
            "cd" => {
                if parts.len() == 1 {
                    env::set_current_dir(env::home_dir().unwrap()).expect("Path not found");
                } else {
                    env::set_current_dir(parts[1]).expect("Path not found");
                }
            }
            "ls" => {
                let mut entries: Vec<_> = fs::read_dir(".").unwrap().map(|e| e.unwrap()).collect();

                entries.sort_by_key(|e| e.file_name().to_string_lossy().to_lowercase());

                for entry in entries {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let entry_metadata = entry.metadata().unwrap();
                    let is_dir = entry_metadata.is_dir();
                    let is_exec = entry_metadata.permissions().mode();

                    if name.starts_with('.') {
                        continue;
                    }

                    if is_dir {
                        print!("\x1b[32m{}\x1b[0m/  ", name);
                    } else if is_exec & 0o111 != 0 {
                        print!("\x1b[34m{}\x1b[0m*  ", name)
                    } else {
                        print!("{}  ", name);
                    }
                }
                println!();
            }
            "exit" => break,
            _ => {
                let result = Command::new(parts[0]).args(&parts[1..]).spawn();

                match result {
                    Ok(mut child) => match child.wait() {
                        Ok(_) => {}
                        Err(e) => eprintln!("Error waiting for command: {}", e),
                    },
                    Err(_) => {
                        println!("{}: command not found", parts[0]);
                    }
                }
            }
        }
    }
}

fn short_path(path: &std::path::Path) -> String {
    let home = env::home_dir().unwrap();
    let path_str = path.to_string_lossy();

    if path_str.starts_with(home.to_string_lossy().as_ref()) {
        path_str.replacen(home.to_string_lossy().as_ref(), "~", 1)
    } else {
        path_str.to_string()
    }
}
