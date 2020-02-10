mod api;
mod game;
mod server;
mod host;
mod benchmark;
mod util;
mod brain;
use clap::{App, Arg, SubCommand};
use util::init_logger;

#[tokio::main]
async fn main() {
    init_logger();

    let matches = App::new("BookWorm")
        .subcommand(SubCommand::with_name("server")
            .about("Run in snake server mode for use by a Battlesnake engine.")
            .arg(Arg::with_name("port")
                .short("p")
                .help("HTTP port to listen on")
                .takes_value(true)
                .default_value("8080")
            )
            .arg(Arg::with_name("ip")
                .short("i")
                .help("Local IP to bind to")
                .takes_value(true)
                .default_value("127.0.0.1")
            )
            .arg(Arg::with_name("budget")
                .short("b")
                .help("Time budget for responding to /move requests, in milliseconds")
                .takes_value(true)
                .default_value("200")
            )
        )
        .subcommand(SubCommand::with_name("host")
            .about("Host a match between snakes.")
            .arg(Arg::with_name("timeout")
                .short("t")
                .help("How long in milliseconds to wait for snake responses")
                .takes_value(true)
                .default_value("500")
            )
            .arg(Arg::with_name("prompt")
                .short("p")
                .help("Prompt for user input to continue each turn of the game")
                .takes_value(false)
                .required(false)
            )
            .arg(Arg::with_name("width")
                .short("w")
                .help("Width of the game board")
                .takes_value(true)
                .default_value("11")
            )
            .arg(Arg::with_name("height")
                .short("h")
                .help("Height of the game board")
                .takes_value(true)
                .default_value("11")
            )
            .arg(Arg::with_name("snake")
                .short("s")
                .help("API endpoint URL(s) of participant snakes")
                .takes_value(true)
                .multiple(true)
                .default_value("localhost:8080")
            )
        )
        .subcommand(SubCommand::with_name("benchmark")
            .about("Execute a series of performance tests, logging results.")
        )
        .get_matches();

    match matches.subcommand() {
        ("server", Some(args)) => {
            server::start_server(
                args.value_of("ip").unwrap().parse().expect("IP must be an IPV6 or IPV4 format"),
                args.value_of("port").unwrap().parse().expect("Port must be numeric"),
                args.value_of("budget").unwrap().parse().expect("Time budget must be numeric")
            ).await;
        }
        ("host", Some(args)) => {
            host::run_game(
                args.value_of("timeout").unwrap().parse().expect("Timeout must be numeric"),
                &args.values_of("snake").expect("At least one snake is needed").map(String::from).collect::<Vec<_>>(),
                args.value_of("width").unwrap().parse().expect("Width must be numeric"),
                args.value_of("height").unwrap().parse().expect("Height must be numeric"),
                args.is_present("prompt")
            ).await;
        }
        ("benchmark", _) => {
            benchmark::run_benchmark();
        }
        _ => {
            eprintln!("Unknown subcommand!");
        }
    }
}
