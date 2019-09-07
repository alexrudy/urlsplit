#![warn(clippy::all)]

use csv;
use docopt;
use docopt::Docopt;
use serde_derive::Deserialize;

use std::error;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

mod delimiter;
mod split;

use delimiter::Delimiter;
use split::OptionDeref;

static USAGE: &str = "
Accepts a newline separated list of URLs and emits a CSV of component parts.

When no input is provided, or input is \"-\", inputs will be read from stdin.
Output is sent to stdout unless the `-o` flag is provided.

The component parts of a URL are split as follows:
    - url: The full input URL.
    - scheme: Identifies the method for loacting this reference. e.g. `http://`
    - host: Where to find this authority, e.g. `example.com` or `my.example.com`
    - path: Within the authority, where to find a resource, e.g. `/path/to/resouce`
    - query: Parameters added to the URL to specify the page content, e.g. `?foo=bar`
    - fragment: Anchor on the page to find the content, e.g. `#some-heading
    - hostname: If the `host` above is a registered name, this contains the full name.
    - domain: The part of the name before the suffix, e.g. `example` for `my.example.com`
    - subdomain: The part of the name which isn' tregistered, e.g. `my` for `my.example.com`
    - suffix: The top level suffix, e.g. `com` or `co.uk`
    - registration: The suffix and domain, combined, e.g. `example.com` for `my.exmaple.com`
    - error: A message describing errors, if any, encourtered while processing this URL.

When the error field is provided, it is text which describes the error encountered
splitting the URL into parts. Some fields may be present when the error field is
not empty, due to the incremental parsing of URLs.

The fields `domain`, `subdomain`, `suffix` and `registration` are derived from the
hostname using the public suffix list (PSL) as implemented in the `tldextract` crate.

Usage:
    urlsplit [options] [<input>]
    urlsplit --help

Common options:
    -h, --help             Display this message
    -o, --output <file>    Write output to <file> instead of stdout.
    -n, --no-headers       When set, the first row emitted will not contain
                           headers, and the input is assumed to not contain headers.
    -d, --delimiter <arg>  The field delimiter for writing CSV data.
                           Must be a single character. (default: ,)
    -q, --quote            When set, enables CSV-style quoting when reading in URLs.

";

type Error = Box<dyn error::Error + 'static>;
type BoxWriter = Box<dyn io::Write + 'static>;
type BoxReader = Box<dyn io::Read + 'static>;

#[derive(Deserialize)]
struct Args {
    arg_input: Option<String>,
    flag_no_headers: bool,
    flag_output: Option<String>,
    flag_delimiter: Option<Delimiter>,
    flag_quote: bool,
}

fn handle_io_path(arg: &Option<String>) -> Option<PathBuf> {
    match arg.as_deref() {
        Some("-") => None,
        Some(s) => Some(PathBuf::from(s)),
        None => None,
    }
}

impl Args {
    fn get_input(&self) -> Option<PathBuf> {
        handle_io_path(&self.arg_input)
    }

    fn get_output(&self) -> Option<PathBuf> {
        handle_io_path(&self.flag_output)
    }

    fn get_headers(&self) -> bool {
        !self.flag_no_headers
    }

    fn get_delimiter(&self) -> Option<u8> {
        self.flag_delimiter.map(|d| d.0)
    }

    fn get_quoting(&self) -> bool {
        self.flag_quote
    }
}

fn ioreader(input: Option<PathBuf>) -> io::Result<BoxReader> {
    Ok(match input {
        None => Box::new(io::stdin()),
        Some(ref p) => match fs::File::open(p) {
            Ok(x) => Box::new(x),
            Err(err) => {
                let msg = format!("failed to open {}: {}", p.display(), err);
                return Err(io::Error::new(io::ErrorKind::NotFound, msg));
            }
        },
    })
}

fn iowriter(output: Option<PathBuf>) -> io::Result<BoxWriter> {
    Ok(match output {
        None => Box::new(io::stdout()),
        Some(ref p) => Box::new(fs::File::create(p)?),
    })
}

fn writer(args: &Args) -> io::Result<csv::Writer<BoxWriter>> {
    let iowriter = iowriter(args.get_output())?;

    let mut builder = csv::WriterBuilder::new();

    if let Some(d) = args.get_delimiter() {
        builder.delimiter(d);
    }

    if !args.get_quoting() {
        builder.quote_style(csv::QuoteStyle::Never);
    }

    Ok(builder.from_writer(iowriter))
}

fn reader(args: &Args) -> io::Result<csv::Reader<BoxReader>> {
    let mut builder = csv::ReaderBuilder::new();

    if let Some(d) = args.get_delimiter() {
        builder.delimiter(d);
    }

    builder.quoting(args.get_quoting());

    builder.has_headers(args.get_headers());
    Ok(builder.from_reader(ioreader(args.get_input())?))
}

fn run(args: Args) -> Result<(), Error> {
    let mut rdr = reader(&args)?;

    let mut wtr = writer(&args)?;

    if args.get_headers() {
        wtr.write_record(&split::header_record())?;
    }

    let mut buf = csv::StringRecord::new();

    while rdr.read_record(&mut buf)? {
        let record = split::parse_url(buf.get(0).unwrap());
        wtr.write_record(&record)?;
    }
    Ok(())
}

fn main() {
    let args: Args = match Docopt::new(USAGE)
        .and_then(|d| d.parse())
        .and_then(|d| d.deserialize())
    {
        Ok(a) => a,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    if let Err(err) = run(args) {
        eprintln!("error parsing URLs: {}", err);
        process::exit(1);
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_handle_io_path() {
        assert_eq!(
            handle_io_path(&Some("Hello".to_string())),
            Some("Hello".into())
        );
    }

}
