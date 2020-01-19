mod api;
mod game;
mod server;
mod benchmark;

use clap::{App, Arg, SubCommand};

fn main() {
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
