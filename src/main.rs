#![feature(plugin)]
#![plugin(docopt_macros)]
#![plugin(regex_macros)]

extern crate rustc_serialize;
extern crate regex;

mod worker;

use worker::cli::Args;
use worker::cli;

// TODO tests.
fn main() {
    let args: Args = cli::parse();
    match args {
        Args { cmd_extract_file: true, .. } => cli::extract_file(&args),
        _ => ()
    }
}
