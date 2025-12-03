#[allow(unused_imports)]
use std::io::{self, Write};
use std::{env, fs, os::unix::fs::PermissionsExt};

fn main() {
    while true {
        let commands = vec!["echo", "type", "exit"];
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let parts = command.split_whitespace().collect::<Vec<&str>>();

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
            },
            "exit" => break,
            _ => println!("{}: command not found", command.trim()),
        }
    }
}
