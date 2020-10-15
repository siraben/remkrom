# remkrom - reimplementation of mkrom in Rust

Writes individual files into a ROM image.  Works as a drop-in
replacement for [mkrom](https://github.com/KnightOS/mkrom/).

## Usage
```
remkrom 0.1.0
A reimplementation of the KnightOS mkrom tool in Rust.

USAGE:
    remkrom <output> <length> [input-files]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <output>            Output file
    <length>            Length of output file in bytes
    <input-files>...    Pairs of the form filename:address
```
