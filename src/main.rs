/*
Copyright (C) 2024  Carl-Philip Hänsch

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
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

// state decode cache to improve state printing speed
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

fn bit_compact_2(a: u64) -> u64 { // compact 64 bits in two-groups to 32 bit (using the lower pairs)
	let b = ((a & 0b0011001100110011001100110011001100110011001100110011001100110000) >>  2) | (a & 0b0000001100110011001100110011001100110011001100110011001100110011);
	let c = ((b & 0b0000111100001111000011110000111100001111000011110000111100000000) >>  4) | (b & 0b0000000000001111000011110000111100001111000011110000111100001111);
	let d = ((c & 0b0000000011111111000000001111111100000000111111110000000000000000) >>  8) | (c & 0b0000000000000000000000001111111100000000111111110000000011111111);
	let e = ((d & 0b0000000000000000111111111111111100000000000000000000000000000000) >> 16) | (d & 0b0000000000000000000000000000000000000000000000001111111111111111);
	return e
}
fn bit_spread_2(a: u64) -> u64 { // spread 32 bits in two-groups
	// 00004321 -> 00430021 -> 04030201
	let b = ((a & 0b0000000000000000000000000000000011111111111111110000000000000000) << 16) | (a & 0b0000000000000000000000000000000000000000000000001111111111111111);
	let c = ((b & 0b0000000000000000111111110000000011111111000000001111111100000000) <<  8) | (b & 0b0000000000000000000000001111111100000000111111110000000011111111);
	let d = ((c & 0b0000000011110000111100001111000011110000111100001111000011110000) <<  4) | (c & 0b0000000000001111000011110000111100001111000011110000111100001111);
	let e = ((d & 0b0000110011001100110011001100110011001100110011001100110011001100) <<  2) | (d & 0b0000001100110011001100110011001100110011001100110011001100110011);
	return e
}

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

	// output functions -> read from state
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

	// rotate left/right (except for accumulator)
	pub fn rol(&mut self) {
		let mut state2: Vec::<u64> = Vec::with_capacity(self.state.capacity());
		let halflen = self.state.len() / 2;
		if halflen == 0 {
			// extra sausage for 6 bit machines
			state2.push(bit_spread_2(self.state[0]) | (bit_spread_2(self.state[0] >> 32) << 2));
		} else {
			for i in 0..halflen {
				let val = self.state[i];
				let val2 = self.state[i+halflen];
				state2.push(bit_spread_2(val      ) | (bit_spread_2(val2      ) << 2));
				state2.push(bit_spread_2(val >> 32) | (bit_spread_2(val2 >> 32) << 2));
			}
		}
		self.state = state2
	}
	pub fn ror(&mut self) {
		let mut state2: Vec::<u64> = Vec::with_capacity(self.state.capacity());
		let halflen = self.state.len() / 2;
		if halflen == 0 {
			// extra sausage for 6 bit machines
			state2.push(bit_compact_2(self.state[0]) | (bit_compact_2(self.state[0] >> 2) << 32));
		} else {
			for i in 0..self.state.len() {
				if i < halflen {
					let val = self.state[2*i];
					let val2 = self.state[2*i+1];
					state2.push(bit_compact_2(val) | (bit_compact_2(val2) << 32));
				} else {
					let val = self.state[2*(i-halflen)];
					let val2 = self.state[2*(i-halflen)+1];
					state2.push(bit_compact_2(val >> 2) | (bit_compact_2(val2 >> 2) << 32));
				}
			}
		}
		self.state = state2
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

	pub fn selectif(&mut self) {
		for v in self.state.iter_mut() {
			*v = *v & 0b1010101010101010101010101010101010101010101010101010101010101010;
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
		// 0001->0001, 0010->0001, 0100->0100, 1000->1000
		for v in self.state.iter_mut() {
			*v = 
				((*v & 0b0001000100010001000100010001000100010001000100010001000100010001)) |
				((*v & 0b0010001000100010001000100010001000100010001000100010001000100010) >> 1) |
				((*v & 0b0100010001000100010001000100010001000100010001000100010001000100)) |
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

	pub fn eq(&mut self) {
		// 0001->0010, 0010->0001, 0100->0100, 1000->1000
		for v in self.state.iter_mut() {
			*v = 
				((*v & 0b0001000100010001000100010001000100010001000100010001000100010001) << 1) |
				((*v & 0b0010001000100010001000100010001000100010001000100010001000100010) >> 1) |
				((*v & 0b0100010001000100010001000100010001000100010001000100010001000100)) |
				((*v & 0b1000100010001000100010001000100010001000100010001000100010001000))
		}
	}

	pub fn imp(&mut self) {
		// 0001->0010, 0010->0010, 0100->0100, 1000->1000
		for v in self.state.iter_mut() {
			*v = 
				((*v & 0b0001000100010001000100010001000100010001000100010001000100010001) << 1) |
				((*v & 0b0010001000100010001000100010001000100010001000100010001000100010)) |
				((*v & 0b0100010001000100010001000100010001000100010001000100010001000100)) |
				((*v & 0b1000100010001000100010001000100010001000100010001000100010001000))
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
		println!(" eq - accumulator := accumulator <=> head");
		println!(" imp - accumulator := accumulator => head (equals !accumulator | head)");
		println!(" not - accumulator := ^accumulator");
		println!(" rol - rotate the stack to the left");
		println!(" ror - rotate the stack to the right");
		println!(" outand - output 1 if all of the accumulators contains a 1");
		println!(" outor - output 1 if any of the accumulators contains a 1");
		println!(" if - only keep those states that have accumulator == 1 (quantum kill switch)");
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
		match cmd.as_str().trim() {
			"reset" => state.reset(),
			"set 0" => state.set0(),
			"set 1" => state.set1(),
			"set x" => state.setx(),
			"read" => state.read(),
			"write" => state.write(),
			"and" => state.and(),
			"or" => state.or(),
			"xor" => state.xor(),
			"eq" => state.eq(),
			"imp" => state.imp(),
			"not" => state.not(),
			"ror" => state.ror(),
			"rol" => state.rol(),
			"outand" => println!("{}", if state.outand() { "1" } else { "0" }),
			"outor" => println!("{}", if state.outor() { "1" } else { "0" }),
			"if" => state.selectif(),
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
