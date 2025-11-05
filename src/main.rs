use crate::file::get_files_to_move;
use crate::model::{print_arguments, validate_arguments, Args};
use chrono::Utc;
use clap::Parser;
use color_eyre::eyre::Result;
use file::{delete_empty_directories, move_files};

mod date;
mod file;
mod log_macro;
mod model;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    validate_arguments(&args)?;
    print_arguments(&args);

    let now = Utc::now();
    let files_to_move = get_files_to_move(&args, now);
    move_files(&args, &files_to_move, args.dry_run)?;
    delete_empty_directories(&args, &args.source)?;

    Ok(())
}