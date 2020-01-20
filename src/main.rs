mod api;
mod game;
mod server;
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
            .about("Runs the bot in server mode for connecting to a Battlesnake engine.")
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
        .subcommand(SubCommand::with_name("benchmark")
            .about("Run a suite of performance tests")
        )
        .get_matches();

    match matches.subcommand() {
        ("server", Some(args)) => {
            server::start_server(
                args.value_of("ip").unwrap().parse().expect("IP must be an IPV6 or IPV4 format"),
                args.value_of("port").unwrap().parse().expect("Port must be numeric")
            );
        },
        ("benchmark", _) => {
            benchmark::run_benchmark();
        },
        _ => {
            println!("Unknown argument!");
        }
    }
}
