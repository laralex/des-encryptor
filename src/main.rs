//! DES - Data Encryption Standard is an old algorithm of symmetric
//! encryption and decryption, operating on 64 bit wide data pieces
//! and applying 56 bit wide keys.
//!
//! Nowdays it’s practical in form of
//! ThreeDES - that is essentially a standard DES applied thrice, or
//! in a form of different modifications of DES (e.g. encrypting using
//! previous blocks or obscured blocks). 
//!
//! Check out rather comprehensive info on Wikipedia
//! https://en.wikipedia.org/wiki/Data_Encryption_Standard
//!
//! [THIS APPLICATION]
//! Is a command line utility for carrying out encryption and
//! decryption of DES and ThreeDES (no modifications are yet
//! implemented). The purpose of this application to exist is my
//! passion to learn Rust (and to use no external libraries).
//!
//! [LICENSE]
//! GNU GPLv3
//! signed, Alexey Larionov, Russia, 2019
//!
//! [USAGE]
//! This is a command line utility (no OS binding, except with
//! possible recompiling). Type --help or -h flag to find out more
//! info

#![allow(dead_code, unused)]
#[macro_use]
extern crate lazy_static;

mod math;
mod des;
mod data_io;
mod reinterpret_bytes;
mod ui;

pub use des::api;
use ui::cli;

fn main() {
    // User interface is acutally an abstraction, currently, the CLI
    // interface is bound in application, but it’s not as hard to make
    // a better one
    use cli::Cli;
    use cli::Action;
    
    let cli = Cli::new()
        .default_key("FFFF-0000-FFFF-0000")
        .expect("Hard coded key is invalid")
        .default_action(cli::Action::EncryptFile)
        .default_endianess(reinterpret_bytes::Endianess::Big)
        .parse_args(std::env::args());

    // Error in the hardcoded build of CLI  
    if let None = cli {
        Cli::print_usage();
        return;
    }
    
    let cli = cli.unwrap();

    // Show help, if --help flag is present, no matter what are other
    // flags and command line arguments
    if !cli.help_requested {
        cli.announce_begin();

        // Open read file, open / create write file
        let (read, write) = data_io::open_rw_files(
            cli.src_file_path(),
            cli.dst_file_path()
        ).expect("Failed I/O operation.");

        match cli.action() {
            Action::EncryptFile => des::api::encrypt(read, write, cli.key(), cli.endianess()),
            Action::DecryptFile => des::api::decrypt(read, write, cli.key(), cli.endianess()),
            Action::TripleEncryptFile => des::api::triple_encrypt(
                read, write, (cli.key(), cli.key(), cli.key()), cli.endianess()),
            Action::TripleDecryptFile => des::api::triple_decrypt(
                read, write, (cli.key(), cli.key(), cli.key()), cli.endianess()),
        }.expect("Internal error (I/O)");
        
        cli.announce_end();    
    } else {
        Cli::print_help();
    }
    
}
