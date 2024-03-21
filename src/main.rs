extern crate clap;

use clap::Parser;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Don't print anything but the output
    #[arg(short, long)]
    quiet: bool,

    /// Stack size in bits
    #[arg(short, long, default_value_t = 6)]
    bitcount: u32,
}


fn main() {
	let args = Args::parse();

	if !args.quiet {
		println!("Hello World from the first nondeterministic 1 bit CPU!");
		println!("");
	}
}
