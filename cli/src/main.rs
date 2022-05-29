pub mod parsing;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|x| x == "-h" || x == "--help") {
        usage(args.get(1));
        std::process::exit(1);
    }

    match args.get(1) {
        Some(command) => match command.as_str() {
            "validate" => validate(&args[2..]),
            _ => main_usage(),
        },
        None => main_usage(),
    }
}

fn validate(args: &[String]) {
    use crate::parsing::parse;
    use std::io::Read;

    let prefer_stdin = args.iter().any(|x| x == "--stdin");
    let verbose = args.iter().any(|x| x == "-v" || x == "--verbose");

    let project_dir_index = args.iter().position(|x| x == "-w" || x == "--workdir");
    let project_dir = match project_dir_index {
        None => std::env::current_dir()
            .map(|path| path)
            .map_err(|_| "Current directory could not be detected"),
        Some(index) => match args.get(index + 1) {
            Some(path_str) => Ok(std::path::Path::new(path_str).to_path_buf()),
            None => Err("Working directory path should be defined!"),
        },
    };

    if let Err(message) = project_dir {
        println!("Error: {message}");
        return;
    }

    let mut buffer = String::new();
    if !prefer_stdin {
        let readers =
            std::fs::read_dir(project_dir.unwrap())
                .unwrap()
                .filter(|entry| match entry {
                    Err(_) => false,
                    Ok(entry) => match entry.path().extension() {
                        None => false,
                        Some(ext) => ext.to_str() == Some("land"),
                    },
                });

        for entry in readers {
            let entry = entry.unwrap();
            let mut file = std::fs::File::open(entry.path()).unwrap();
            match file.read_to_string(&mut buffer) {
                Ok(_) => match parse(&buffer) {
                    Ok(tree) => {
                        if verbose {
                            println!("{tree:#?}");
                        }
                        println!("Source code is OK");
                    }
                    Err(error) => println!("Parse Error: {error}"),
                },
                Err(error) => println!("Stream Error: {error}"),
            }
        }
    } else {
        let mut reader = std::io::stdin();
        match reader.read_to_string(&mut buffer) {
            Ok(_) => match parse(&buffer) {
                Ok(tree) => {
                    if verbose {
                        println!("{tree:#?}");
                    }
                    println!("Source code is OK");
                }
                Err(error) => println!("Parse Error: {error}"),
            },
            Err(error) => println!("Stream Error: {error}"),
        }
    }
}

fn usage(command: Option<&String>) {
    match command {
        Some(command) => match command.as_str() {
            "validate" => validate_usage(),
            _ => main_usage(),
        },
        None => main_usage(),
    }
}

fn main_usage() {
    print!(
        r###"landlord - version {VERSION:?}

Better terraformation tooling

USAGE:
    landlord <command> [options]

OPTIONS:
    -h, --help  Print help information

COMMANDS:
    validate    Validate given source code
"###
    );
}

fn validate_usage() {
    print!(
        r###"landlord validate - version {VERSION:?}

Validate given source code

USAGE:
    landlord validate [options]

OPTIONS:
    -h, --help              Print help information
        --stdin             Prefer taking source code from standard input
    -w, --workdir <path>    Set working directory to validate (default: current directory)
    -v, --verbose           Print syntax tree
"###
    );
}
