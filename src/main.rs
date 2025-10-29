use std::{borrow::Cow, collections::HashMap, net::Ipv4Addr, sync::Arc};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tower_lsp::{LspService, Server};
use tree_sitter::Parser;

mod cli;
mod instructions;
mod server;

use server::{Backend, Configuration};

#[tokio::main]
async fn main() {
    use clap::Parser as _;
    let cli = cli::Cli::parse();

    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_ic10::language())
        .expect("Failed to set language");

    let (service, socket) = LspService::new(|client| Backend {
        client,
        files: Arc::new(RwLock::new(HashMap::new())),
        config: Arc::new(RwLock::new(Configuration::default())),
    });

    if !cli.listen && cli.host.is_none() {
        // stdin/stdout
        Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
            .serve(service)
            .await;
    } else if cli.listen {
        // listen

        let host = cli
            .host
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed("127.0.0.1"))
            .parse::<Ipv4Addr>()
            .expect("Could not parse IP address");

        let port = cli.port.unwrap_or(9257);

        let stream = {
            let listener = TcpListener::bind((host, port)).await.unwrap();
            let (stream, _) = listener.accept().await.unwrap();
            stream
        };

        let (input, output) = tokio::io::split(stream);
        Server::new(input, output, socket).serve(service).await;
    } else {
        let host = cli.host.expect("No host given");
        let port = cli.port.expect("No port given");

        let stream = TcpStream::connect((host, port))
            .await
            .expect("Could not open TCP stream");

        let (input, output) = tokio::io::split(stream);
        Server::new(input, output, socket).serve(service).await;
    }
}
