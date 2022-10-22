# Compiler
This compiler is written in Rust programming language. It uses the [LALRPOP](https://github.com/lalrpop/lalrpop) for generating a parser. The file `src/grammar.lalrpop` contains the grammar used by the `LALRPOP` to generate code during compilation (see `build.rs`).

Command `make` produces an executable `instant` in the root of the project. To compile the project, `make` uses `cargo` - the standard package manager for Rust.

`instant` reads a program in the Instant language from the STDIN and outputs the compiled code to STDOUT.

# Dependencies
* [Jasmin](http://jasmin.sourceforge.net/) - as `.jar` used for JVM bytecode generation.
* [LALRPOP](https://github.com/lalrpop/lalrpop) - Rust package for parser generation.
