
mod server;
mod client;
mod logic;
mod app;
mod ui;
mod tcp_thread;

use server::Server;
use client::Client;
use logic;
use ui;
use app;
use std::env;
use std::process;

const PORT: &str = "8384";

fn main() {

    let args: Vec<_> = env::args().collect();
    let name = args[0];

    let layer = match args.len() {

        1 => err_exit(name, format!("Missing option")),

        _ => match args[1].as_str() {

            "serve" => Server::new(PORT.to_string()),

            "connect" => {

                if args.len() == 3 {
                    Client::new(format!("{}:{}", args[2], PORT))
                } else {
                    err_exit(name, "Missing address".to_string())
                }
            },

            _ => err_exit(name, format!("Unknown option: {}", args[1])),
        },
    };

    app::run(layer);    
}

fn usage(name: String) {

    println!("
        Usage:
            {0} serve           Start server.
            {0} connect <addr>  Connect to server at address <addr>.
    ", name);
}

fn err_exit(name: String, msg: String) -> ! {

    println!("ERROR: {}", msg);
    usage(name);
    process::exit(1)
}
