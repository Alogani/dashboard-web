use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file
    #[arg(short, long)]
    pub config: PathBuf,

    /// Manage users (add/update passwords)
    #[arg(short, long)]
    pub manage_users: bool,
}
