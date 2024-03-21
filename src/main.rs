extern crate clap;

use clap::Parser;
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Don't print anything but the output
    #[arg(short, long)]
    quiet: bool,

    /// Stack size in bits
    #[arg(short, long, default_value_t = 6)]
    bitcount: usize,
}

static HEADS: &'static [&'static str] = &[
"000000",
"000001",
"000010",
"000011",
"000100",
"000101",
"000110",
"000111",
"001000",
"001001",
"001010",
"001011",
"001100",
"001101",
"001110",
"001111",
"010000",
"010001",
"010010",
"010011",
"010100",
"010101",
"010110",
"010111",
"011000",
"011001",
"011010",
"011011",
"011100",
"011101",
"011110",
"011111",
"100000",
"100001",
"100010",
"100011",
"100100",
"100101",
"100110",
"100111",
"101000",
"101001",
"101010",
"101011",
"101100",
"101101",
"101110",
"101111",
"110000",
"110001",
"110010",
"110011",
"110100",
"110101",
"110110",
"110111",
"111000",
"111001",
"111010",
"111011",
"111100",
"111101",
"111110",
"111111",
];

pub struct State {
	bitsize: usize,
	state: Vec<u64>,
}

impl State {
	pub fn new(bitsize: usize) -> Self {
		let sz = 1 << (bitsize - 6);
		let mut result = Self { bitsize: bitsize, state: Vec::with_capacity(sz) };
		for _ in 0..sz {
			result.state.push(0)
		}
		return result
	}

	pub fn print(&self) {
		for (i, val) in self.state.iter().enumerate() {
			let pfx: String = format!("{i:064b}").chars().skip(64+6-self.bitsize).collect();
			for b in 0..64 {
				if (val & (1 << b)) != 0 {
					println!("{}{}", pfx, HEADS[b]);
				}
			}
		}
	}

	pub fn reset(&mut self) {
		for v in self.state.iter_mut() {
			*v = 0;
		}
		self.state[0] = 1;
	}

	pub fn set0(&mut self) {
		for v in self.state.iter_mut() {
			*v = ((*v & 0b1010101010101010101010101010101010101010101010101010101010101010) >> 1) | (*v & 0b0101010101010101010101010101010101010101010101010101010101010101);
		}
	}

	pub fn set1(&mut self) {
		for v in self.state.iter_mut() {
			*v = (*v & 0b1010101010101010101010101010101010101010101010101010101010101010) | ((*v & 0b0101010101010101010101010101010101010101010101010101010101010101) << 1);
		}
	}

	pub fn setx(&mut self) {
		for v in self.state.iter_mut() {
			*v = ((*v & 0b1010101010101010101010101010101010101010101010101010101010101010) >> 1) | (*v & 0b0101010101010101010101010101010101010101010101010101010101010101)
			    | (*v & 0b1010101010101010101010101010101010101010101010101010101010101010) | ((*v & 0b0101010101010101010101010101010101010101010101010101010101010101) << 1);
		}
	}
}


fn main() {
	let args = Args::parse();

	if !args.quiet {
		println!("Hello World from the first nondeterministic 1 bit CPU!");
		println!("");
		println!("You can use the following commands:");
		println!(" reset - resets the machine to a predefined state");
		println!(" set 0 - sets the top bit of the stack to 0");
		println!(" set 1 - sets the top bit of the stack to 1");
		println!(" set x - sets the top bit of the stack to 0 and 1 simultaneously");
		println!("");
	}

	let mut state = State::new(args.bitcount);
	state.reset();

	if !args.quiet {
		println!("Starting with initial state:");
		state.print();
		println!("");
	}

	let stdin = io::stdin();
	for cmd_result in stdin.lock().lines() {
		let cmd = cmd_result.unwrap();
		match cmd.as_str() {
			"reset" => state.reset(),
			"set 0" => state.set0(),
			"set 1" => state.set1(),
			"set x" => state.setx(),
			_ => println!("unknown command: {cmd}"),
		}

		// print state after each command
		state.print();
		println!("");
	}
}
