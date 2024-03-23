Nondeterministic CPU
====================

Here we present the first nondeterministic CPU. You can run it with any bitsize you want. A 32 bit nondeterministic CPU has the equivalent performance of a <b>4 billion core CPU</b>.

It has a word size of 1 bit, one accumulator register and a stack of as much bits as you want. It does not have a instruction memory - all instructions are fed from the outside.

This CPU can solve NP hard problems in P-TIME steps. It may be very efficient for brute forcing NP hard problems.

[![ndcpu demo](https://img.youtube.com/vi/31zXnuZ_dFA/0.jpg)](https://www.youtube.com/watch?v=31zXnuZ_dFA)

Quantum Computing with nondeterministic CPUs
---

A nondeterministic CPU has the same properties as a quantum processor: it can solve NP hard problems in polynomial time.

In this implementation, "polynomial time" is relative. The computation takes a polynomial amount of calculation steps. However we have to implement the ndcpu on a deterministic machine, so the current implementation on a deterministic machine involves exponentially many computation cycles per calculation step. With a few tweaks like bit vector arithmetic and a fixed limited machine size, we reach a decent amount of nondeterministic computation power.

Compiling and running
---

ndcpu is implemented in rust. Be sure to install `rustc` and `cargo` before compiling:
```bash
git clone https://github.com/carli2/ndcpu
cd ndcpu
make && ./ndcpu -b 32
```

Example: Propositional Logic SAT solving
---

Let's examine if `(A => B) <=> (!B => !A)` is a tautology. We need only 3-4 bits, however the minimum number of bits is 6, so we run it with `./ndcpu -q -b 6` in quiet mode:

```
# write A onto stack
set x
write
rol

# write B onto stack
set x
write

# from now the program works in 4 states simultanously

# calculate A => B and put it onto stack
ror
read
rol
imp
rol
write

# Fetch and negate A and write it onto stack
ror
ror
read
not
ror
write

# Go to B, load and negate it
rol
rol
read
not

# now !B is in accumulator, move to !A and imply
ror
ror
imp

# now move to A => B and calculate <=>
rol
rol
rol
eq

# output the result
outand
```

The result is `1`, so the formula is a tautology.


Implementation Details
---
A single state of a nondeterministic CPU can be just encoded as an array of bits to represent the state. To encode all states, we initialize a bitvector and use the state as the address to a bit.
On initialization, we set the bitvector to zero except for the 000000-state (which is the least significant bit in our bitvector) which will be turned `1`.

Whenever there is a operation on the state vector, it can be implemented as a set of bit manipulation tricks that I won't explain further but you can check it out in `src/`.

Nondeterministic silicon hardware
---

This nondeterministic CPU can of course be implemented in hardware. The chip would have only 7 pins regardless of the amount of qbits that have been put into this hardware:
 - 4 pins to feed the instruction (there are 16 instructions in the moment)
 - 1 pin for clock: with a raising edge of the clock, the instruction is read and the new state is calculated
 - 1 pin for the result of `outand` (universal quantifier)
 - 1 pin for the result of `outor` (existence quantifier)
 - maybe a ready flag if the result needs more than one cycle

Some implementation hints:
 - when a ndcpu is implemented on pure transistor logic and state flipflops, the amount of qbits might be very low
 - there could be a nondeterministic ALU that gets DRAM bursts fired into it
 - multiple execution units could be fitted into a cluster to reach more qbits
