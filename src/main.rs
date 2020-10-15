use std::error::Error;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::{convert::TryInto, io::prelude::*, num::ParseIntError};
use std::{fs::File, io::SeekFrom};
use structopt::StructOpt;

fn parse_hex(input: &str) -> Result<usize, ParseIntError> {
    match input.strip_prefix("0x") {
        Some(input) => usize::from_str_radix(input, 16),
        None => usize::from_str_radix(input, 16),
    }
}

#[derive(StructOpt)]
struct HexOpt {
    #[structopt(parse(try_from_str = parse_hex))]
    val: usize,
}

impl FromStr for HexOpt {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_hex(s).map(|x| HexOpt { val: x })
    }
}

#[derive(StructOpt)]
#[structopt(
    name = "remkrom",
    about = "A reimplementation of the KnightOS mkrom tool in Rust."
)]
struct Opt {
    /// Output file
    #[structopt(parse(from_os_str))]
    output: PathBuf,

    /// Length of output file in bytes
    length: HexOpt,

    /// Pairs of the form filename:address
    #[structopt(parse(try_from_str = parse_key_val))]
    input_files: Vec<(PathBuf, HexOpt)>,
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s
        .find(':')
        .ok_or_else(|| format!("invalid KEY:value: no `:` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

struct InputFile<'a> {
    file: &'a Path,
    offset: usize,
}

struct Context {
    rom: BufWriter<File>,
}

impl<'a> InputFile<'a> {
    fn new(file: &'a Path, offset: usize) -> InputFile<'a> {
        InputFile { file, offset }
    }
}

impl Context {
    fn new<'a>(rom_path: &'a Path, length: usize) -> Context {
        let mut rom = BufWriter::new(File::create(&rom_path).unwrap());
        rom.write(&vec![0xff; length]).unwrap();
        rom.seek(SeekFrom::Start(0)).unwrap();
        Context { rom }
    }
    fn write_file_to_image(&mut self, input: InputFile) {
        let mut file = match File::open(input.file) {
            Err(why) => panic!("Unable to open {}: {}", input.file.display(), why),
            Ok(file) => BufReader::new(file),
        };

        let mut buf = Vec::new();
        file.seek(SeekFrom::Start(input.offset.try_into().unwrap()))
            .unwrap();

        file.read_to_end(&mut buf).unwrap();
        self.rom.write_all(&mut buf).unwrap();
    }
}

fn main() {
    let opt = Opt::from_args();
    let mut context = Context::new(&opt.output, opt.length.val);
    for (p, HexOpt { val: offset }) in opt.input_files.iter() {
        context.write_file_to_image(InputFile::new(p, *offset))
    }
}
