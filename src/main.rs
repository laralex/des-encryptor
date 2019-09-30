#[macro_use] extern crate lazy_static;

mod des;
mod data_io;
use des::cli;

fn main() {
    use cli::Cli;
    let cli = Cli::new()
        .default_key("FFFF-0000-FFFF-0000")
        .expect("Hard coded key is invalid")
        .default_action(cli::Action::EncryptFile)
        .parse_args(std::env::args());
    
    if let None = cli {
        Cli::print_help();
        return;
    }
    
    let cli = cli.unwrap();
    
    cli.announce_begin();

    let (read, write) = data_io::open_rw_files(
        &cli.src_file_path,
        &cli.dst_file_path
    ).expect("Failed I/O operation.");

    
    des::api::encrypt(read, write, cli.key);
    
    cli.announce_end();
}
