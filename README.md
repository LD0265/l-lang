# L-Lang
## L-Lang stands for "L language" (L as in bad).


Ecc was my first compiler, it wasn't very good so I made a better one.

L-lang is still being worked on and can't do much more than store variables


As in the original, all generated assembly was tested using the [MARS MIPS Assembler](https://computerscience.missouristate.edu/mars-mips-simulator.htm)

## Build and Install
Since this is a rust project you'll need cargo to build and install this

Also when you build, you might get a bunch of warnings, that's okay because this isn't a serious compiler

Run these in your terminal emulator of choice

```bash
git clone https://github.com/LD0265/l-lang.git
cd l-lang
cargo build -r
cargo install --path .
```
