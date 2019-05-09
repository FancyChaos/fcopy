use fcopy::ClipboardManager;
use nix::unistd::{fork, ForkResult};
use clap::{Arg, App};
use std::fs;

// TODO: Get the ICCCM standard as right as possible (or iam willing to do)
// TODO: Support more targets (atoms)
// TODO: Fix the console error (only I know what it means)
// TODO: Add way better error handeling
// TODO: Refactor much of the code to make it waaaay more cleaner (and possible faster)

fn main() {
    // Do arguement parsing and file parsing
    let matches = App::new("fcopy")
                        .version("0.1")
                        .author("Felix Scheja <Felixs.Developer@gmail.com>")
                        .about("Copy the content of a file to the clipboard")
                        .arg(Arg::with_name("INPUT")
                            .help("Input file")
                            .required(true)
                            .index(1))
                        .arg(Arg::with_name("test")
                            .short("t")
                            .long("test")
                            .takes_value(false)
                            .help("Start the application in testing mode (for development)"))
                        .get_matches();

    let file = matches.value_of("INPUT").unwrap();

    if let Ok(data) = parse_file(&file) {
        if matches.occurrences_of("test") > 0 {
            // Testing
            ClipboardManager::new(data, true).run().unwrap();
        } else {
            // Create a new process which will manage the clipboard
            match fork() {
                Ok(ForkResult::Parent{..}) => (),
                Ok(ForkResult::Child) => {
                    ClipboardManager::new(data, false).run().unwrap();
                },
                Err(_) => eprintln!("fcopy Error: Error occured!"),
            } 
        }
    } else {
        eprintln!("fcopy Error: Could not read content of '{}'", file);
        eprintln!("fcopy Error: Does the file exist?");
    }
}

fn parse_file(file: &str) -> Result<String, std::io::Error> {
    // TODO: Take arguements into account
    // like '-n' for lines
    // and a 'grep' like functionality
    fs::read_to_string(file)
}