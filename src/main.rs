use csv;
use docopt;
use docopt::Docopt;
use serde_derive::Deserialize;
use std::error;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process;

mod delimiter;
mod split;

use delimiter::Delimiter;
use split::OptionDeref;

static USAGE: &'static str = "
Accepts a newline separated list of URLs and emits a CSV of URLs split into their component parts.

When no input is provided, or input is \"-\", inputs will be read from stdin.

Usage:
    urlsplit [options] [<input>]
    urlsplit --help

Common options:
    -h, --help             Display this message
    -o, --output <file>    Write output to <file> instead of stdout.
    -n, --no-headers       When set, the first row emitted will not be contain
                           headers.
    -d, --delimiter <arg>  The field delimiter for writing CSV data.
                           Must be a single character. (default: ,)

";

type Error = Box<dyn error::Error + 'static>;

#[derive(Deserialize)]
struct Args {
    arg_input: Option<String>,
    flag_no_headers: bool,
    flag_output: Option<String>,
    flag_delimiter: Option<Delimiter>,
}

impl Args {
    fn get_input(&self) -> Option<PathBuf> {
        match self.arg_input.as_deref() {
            Some("-") => None,
            Some(s) => Some(PathBuf::from(s)),
            None => None,
        }
    }

    fn get_output(&self) -> Option<PathBuf> {
        match self.flag_output.as_deref() {
            Some("-") => None,
            Some(s) => Some(PathBuf::from(s)),
            None => None,
        }
    }

    fn get_headers(&self) -> bool {
        !self.flag_no_headers
    }

    fn get_delimiter(&self) -> Option<u8> {
        self.flag_delimiter.map(|d| d.0)
    }
}

fn reader(input: Option<PathBuf>) -> io::Result<Box<dyn io::BufRead + 'static>> {
    Ok(match input {
        None => Box::new(BufReader::new(io::stdin())),
        Some(ref p) => match fs::File::open(p) {
            Ok(x) => Box::new(BufReader::new(x)),
            Err(err) => {
                let msg = format!("failed to open {}: {}", p.display(), err);
                return Err(io::Error::new(io::ErrorKind::NotFound, msg));
            }
        },
    })
}

fn writer(output: Option<PathBuf>) -> io::Result<Box<dyn io::Write + 'static>> {
    Ok(match output {
        None => Box::new(io::stdout()),
        Some(ref p) => Box::new(fs::File::create(p)?),
    })
}

fn run() -> Result<(), Error> {
    let args: Args = Docopt::new(USAGE)?.parse()?.deserialize()?;

    let mut rdr = reader(args.get_input())?;

    let iowriter = writer(args.get_output())?;
    let mut writer = match args.get_delimiter() {
        Some(d) => csv::WriterBuilder::new().delimiter(d).from_writer(iowriter),
        None => csv::Writer::from_writer(iowriter),
    };

    if args.get_headers() {
        writer.write_record(&split::header_record())?;

        let mut buf = String::new();
        rdr.read_line(&mut buf)?;
    }

    for line in rdr.lines() {
        let record = split::parse_url(&line?.trim_matches('"'));
        writer.write_record(&record)?;
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("error parsing URLs: {}", err);
        process::exit(1);
    };
}
