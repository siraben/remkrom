use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::{error::Error, io};
use std::{fs::File, io::SeekFrom};
use std::{io::prelude::*, num::ParseIntError};
use structopt::StructOpt;

fn parse_hex(input: &str) -> Result<u64, ParseIntError> {
    match input.strip_prefix("0x") {
        Some(input) => u64::from_str_radix(input, 16),
        None => u64::from_str_radix(input, 16),
    }
}

#[derive(StructOpt)]
struct HexOpt {
    #[structopt(parse(try_from_str = parse_hex))]
    val: u64,
}

impl FromStr for HexOpt {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_hex(s).map(|val| HexOpt { val })
    }
}

#[derive(StructOpt)]
#[structopt(name = "remkrom")]
/// A reimplementation of the KnightOS mkrom tool in Rust.
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
    offset: u64,
}

struct Context {
    rom: BufWriter<File>,
}

impl Context {
    fn new<'a>(rom_path: &'a Path, length: u64) -> Result<Context, io::Error> {
        let mut rom = BufWriter::new(File::create(&rom_path)?);
        for _ in 0..length {
            rom.write(&[0xff])?;
        }
        rom.seek(SeekFrom::Start(0))?;
        Ok(Context { rom })
    }
    fn write_file_to_image(&mut self, input: InputFile) -> Result<(), io::Error> {
        let mut file = match File::open(input.file) {
            Err(why) => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("Unable to open {}: {}", input.file.display(), why),
                ))
            }
            Ok(file) => BufReader::new(file),
        };

        let mut buf = vec![];
        file.seek(SeekFrom::Start(input.offset))?;
        file.read_to_end(&mut buf)?;
        self.rom.write_all(&mut buf)
    }
    fn run(&mut self, opt: &Opt) -> Result<(), io::Error> {
        for (file, HexOpt { val }) in opt.input_files.iter() {
            self.write_file_to_image(InputFile {
                file: &file,
                offset: *val,
            })?;
        }
        Ok(())
    }
}

fn main() {
    let opt = Opt::from_args();
    match Context::new(&opt.output, opt.length.val).and_then(|mut c| c.run(&opt)) {
        Ok(()) => std::process::exit(0),
        Err(e) => {
            eprintln!("{}", e.get_ref().unwrap_or(&e));
            std::process::exit(1);
        }
    }
}
