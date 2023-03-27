use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
pub(crate) struct Cli {
    #[arg(long)]
    pub listen: bool,
    pub host: Option<String>,
    pub port: Option<u16>,
}
