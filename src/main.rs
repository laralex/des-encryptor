#[macro_use]
extern crate lazy_static;

mod math;
mod des;
mod data_io;
mod reinterpret_bytes;
mod ui;
use ui::cli;

pub use des::api;

fn main() {
    use cli::Cli;
    let cli = Cli::new()
        .default_key("FFFF-0000-FFFF-0000")
        .expect("Hard coded key is invalid")
        .default_action(cli::Action::EncryptFile)
        .default_endianess(reinterpret_bytes::Endianess::Big)
        .parse_args(std::env::args());
    
    if let None = cli {
        Cli::print_usage();
        return;
    }
    
    let cli = cli.unwrap();

    if !cli.help_requested {
        cli.announce_begin();

        let (read, write) = data_io::open_rw_files(
            cli.src_file_path(),
            cli.dst_file_path()
        ).expect("Failed I/O operation.");

        use cli::Action;
        match cli.action() {
            Action::EncryptFile => des::api::encrypt(read, write, cli.key(), cli.endianess()),
            Action::DecryptFile => des::api::decrypt(read, write, cli.key(), cli.endianess()),
        }.expect("Internal error (I/O)");
        
        cli.announce_end();    
    } else {
        Cli::print_help();
    }
    
}
