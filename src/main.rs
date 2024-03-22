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
			if *val != 0 {
				let pfx: String = format!("{i:064b}").chars().skip(64+6-self.bitsize).collect();
				for b in 0..64 {
					if (val & (1 << b)) != 0 {
						println!("{}{}", pfx, HEADS[b]);
					}
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

	pub fn outand(&mut self) -> bool {
		for v in self.state.iter() {
			// if there is a zero inside this array, fail!
			if (*v & 0b0101010101010101010101010101010101010101010101010101010101010101) != 0 {
				return false
			}
		}
		return true
	}

	pub fn outor(&mut self) -> bool {
		for v in self.state.iter() {
			// if there is a one inside this array, succeed
			if (*v & 0b1010101010101010101010101010101010101010101010101010101010101010) != 0 {
				return true
			}
		}
		return false
	}

	// accumulator only functions
	// 0: *v & 0b0101010101010101010101010101010101010101010101010101010101010101
	// 1: *v & 0b1010101010101010101010101010101010101010101010101010101010101010
	pub fn set0(&mut self) {
		// 00->00, 01->01, 10->01, 11->01
		for v in self.state.iter_mut() {
			*v = ((*v & 0b1010101010101010101010101010101010101010101010101010101010101010) >> 1) | (*v & 0b0101010101010101010101010101010101010101010101010101010101010101);
		}
	}

	pub fn set1(&mut self) {
		// 00->00, 01->10, 10->10, 11->10
		for v in self.state.iter_mut() {
			*v = (*v & 0b1010101010101010101010101010101010101010101010101010101010101010) | ((*v & 0b0101010101010101010101010101010101010101010101010101010101010101) << 1);
		}
	}

	pub fn setx(&mut self) {
		// 00->00, 01->11, 10->11, 11->11
		for v in self.state.iter_mut() {
			*v = ((*v & 0b1010101010101010101010101010101010101010101010101010101010101010) >> 1) | (*v & 0b0101010101010101010101010101010101010101010101010101010101010101)
			    | (*v & 0b1010101010101010101010101010101010101010101010101010101010101010) | ((*v & 0b0101010101010101010101010101010101010101010101010101010101010101) << 1);
		}
	}

	pub fn not(&mut self) {
		// 00->00, 01->10, 10->01, 11->11
		for v in self.state.iter_mut() {
			*v = ((*v & 0b1010101010101010101010101010101010101010101010101010101010101010) >> 1) | ((*v & 0b0101010101010101010101010101010101010101010101010101010101010101) << 1);
		}
	}

	// accumulator+head functions
	// 00: *v & 0b0001000100010001000100010001000100010001000100010001000100010001
	// 01: *v & 0b0010001000100010001000100010001000100010001000100010001000100010
	// 10: *v & 0b0100010001000100010001000100010001000100010001000100010001000100
	// 11: *v & 0b1000100010001000100010001000100010001000100010001000100010001000
	pub fn write(&mut self) {
		// 0001->0001, 0010->1000, 0100->0001, 1000->1000
		for v in self.state.iter_mut() {
			*v = 
				((*v & 0b0001000100010001000100010001000100010001000100010001000100010001)) |
				((*v & 0b0010001000100010001000100010001000100010001000100010001000100010) << 2) |
				((*v & 0b0100010001000100010001000100010001000100010001000100010001000100) >> 2) |
				((*v & 0b1000100010001000100010001000100010001000100010001000100010001000))
		}
	}

	pub fn read(&mut self) {
		// 0001->0001, 0010->0001, 0100->1000, 1000->1000
		for v in self.state.iter_mut() {
			*v = 
				((*v & 0b0001000100010001000100010001000100010001000100010001000100010001)) |
				((*v & 0b0010001000100010001000100010001000100010001000100010001000100010) >> 1) |
				((*v & 0b0100010001000100010001000100010001000100010001000100010001000100) << 1) |
				((*v & 0b1000100010001000100010001000100010001000100010001000100010001000))
		}
	}

	pub fn and(&mut self) {
		// 0001->0001, 0010->0001, 0100->0001, 1000->1000
		for v in self.state.iter_mut() {
			*v = 
				((*v & 0b0001000100010001000100010001000100010001000100010001000100010001)) |
				((*v & 0b0010001000100010001000100010001000100010001000100010001000100010) >> 1) |
				((*v & 0b0100010001000100010001000100010001000100010001000100010001000100) >> 2) |
				((*v & 0b1000100010001000100010001000100010001000100010001000100010001000))
		}
	}

	pub fn or(&mut self) {
		// 0001->0001, 0010->0010, 0100->1000, 1000->1000
		for v in self.state.iter_mut() {
			*v = 
				((*v & 0b0001000100010001000100010001000100010001000100010001000100010001)) |
				((*v & 0b0010001000100010001000100010001000100010001000100010001000100010)) |
				((*v & 0b0100010001000100010001000100010001000100010001000100010001000100) << 1) |
				((*v & 0b1000100010001000100010001000100010001000100010001000100010001000))
		}
	}

	pub fn xor(&mut self) {
		// 0001->0001, 0010->0010, 0100->1000, 1000->0100
		for v in self.state.iter_mut() {
			*v = 
				((*v & 0b0001000100010001000100010001000100010001000100010001000100010001)) |
				((*v & 0b0010001000100010001000100010001000100010001000100010001000100010)) |
				((*v & 0b0100010001000100010001000100010001000100010001000100010001000100) << 1) |
				((*v & 0b1000100010001000100010001000100010001000100010001000100010001000) >> 1)
		}
	}
}


fn main() {
	let mut args = Args::parse();

	if !args.quiet {
		println!("Hello World from the first nondeterministic 1 bit CPU!");
		println!("");
		println!("You can use the following commands:");
		println!(" reset - resets the machine to a predefined state");
		println!(" set 0 - accumulator := 0");
		println!(" set 1 - accumulator := 1");
		println!(" set x - accumulator := 0 and 1 simultaneously");
		println!(" write - head := accumulator");
		println!(" read - accumulator := head");
		println!(" and - accumulator := accumulator & head");
		println!(" or - accumulator := accumulator | head");
		println!(" xor - accumulator := accumulator ^ head");
		println!(" not - accumulator := ^accumulator");
		println!(" lrot - rotate the stack to the left");
		println!(" rrot - rotate the stack to the right");
		println!(" outand - output 1 if all of the accumulators contains a 1");
		println!(" outor - output 1 if any of the accumulators contains a 1");
		println!(" quiet - turn on quiet mode");
		println!(" !quiet - turn off quiet mode");
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
			"read" => state.read(),
			"write" => state.write(),
			"and" => state.and(),
			"or" => state.or(),
			"xor" => state.xor(),
			"not" => state.not(),
			"outand" => println!("{}", if state.outand() { "1" } else { "0" }),
			"outor" => println!("{}", if state.outor() { "1" } else { "0" }),
			"quiet" => args.quiet = true,
			"!quiet" => args.quiet = false,
			_ => println!("unknown command: {cmd}"),
		}

		// print state after each command
		if !args.quiet {
			state.print();
			println!("");
		}
	}
}
