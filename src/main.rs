mod api;
mod game;
mod server;
mod host;
mod benchmark;

use log::*;
use clap::{App, Arg, SubCommand};
use chrono::{DateTime, Local};

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let now: DateTime<Local> = Local::now();
        println!("{} [{}] {}", now.format("%Y-%m-%d %H:%M:%S%.6f"), record.level(), record.args());
    }

    fn flush(&self) {}
}

const LOGGER: Logger = Logger;

fn main() {
    if set_logger(&LOGGER).is_ok() {
        log::set_max_level(LevelFilter::Debug)
    }

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
        )
        .subcommand(SubCommand::with_name("host")
            .about("Host a match between snakes.")
            .arg(Arg::with_name("timeout")
                .short("t")
                .help("How long in milliseconds to wait for snake responses")
                .takes_value(true)
                .default_value("500")
            )
            .arg(Arg::with_name("width")
                .short("w")
                .help("Width of the game board")
                .takes_value(true)
                .default_value("12")
            )
            .arg(Arg::with_name("height")
                .short("h")
                .help("Height of the game board")
                .takes_value(true)
                .default_value("12")
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
                args.value_of("port").unwrap().parse().expect("Port must be numeric")
            );
        }
        ("host", Some(args)) => {
            host::run_game(
                args.value_of("timeout").unwrap().parse().expect("Timeout must be numeric"),
                args.values_of("snake").expect("At least one snake is needed").map(String::from).collect(),
                args.value_of("width").unwrap().parse().expect("Width must be numeric"),
                args.value_of("height").unwrap().parse().expect("Height must be numeric")
            );
        }
        ("benchmark", _) => {
            benchmark::run_benchmark();
        }
        _ => {
            eprintln!("Unknown subcommand!");
        }
    }
}
