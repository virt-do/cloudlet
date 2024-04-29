use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ApiArgs {
    #[arg(short, long)]
    pub config_path: Option<PathBuf>
}