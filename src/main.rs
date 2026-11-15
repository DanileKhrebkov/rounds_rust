use bevy::prelude::*;
use std::env;

mod shared;
mod client;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--server") {
        println!("Starting as SERVER on port {}", shared::SERVER_PORT);
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(server::ServerPlugin)
            .run();
    } else if let Some(pos) = args.iter().position(|a| a == "--client") {
        let server_ip = args
            .get(pos + 1)
            .expect("Usage: cargo run -- --client <SERVER_IP>");
        println!("Starting as CLIENT, connecting to {server_ip}");
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(client::ClientPlugin {
                server_addr: format!("{}:{}", server_ip, shared::SERVER_PORT),
            })
            .run();
    } else {
        eprintln!("Usage:");
        eprintln!("  cargo run -- --server");
        eprintln!("  cargo run -- --client <SERVER_IP>");
    }
}