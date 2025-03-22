use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("rust-my")
        .version("0.1.0")
        .author("Daniel Echlin")
        .about("Your custom CLI tool")
        .subcommand(
            SubCommand::with_name("greet")
                .about("Greets a person")
                .arg(
                    Arg::with_name("name")
                        .help("The name of the person to greet")
                        .required(false)
                        .index(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("greet", greet_matches)) => {
            let name = greet_matches.value_of("name").unwrap_or("friend");
            println!("Hello, {}!", name);
        }
        _ => println!("Try using a subcommand like 'greet'"),
    }
}