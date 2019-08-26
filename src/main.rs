use csv;
use std::error::Error;
use std::io;
use std::process;
use url::Url;

fn main() {
    if let Err(err) = split_urls() {
        println!("error parsing URLs: {}", err);
        process::exit(1);
    };
}

fn empty_row(index: &str) -> Result<csv::ByteRecord, Box<dyn Error>> {
    let values = vec![index, "", "", "", "", "", ""];
    Ok(csv::ByteRecord::from(values))
}

fn url_row(index: &str, url: Url) -> Result<csv::ByteRecord, Box<dyn Error>> {
    let mut values = Vec::with_capacity(7);

    values.push(index);
    values.push(url.scheme());
    values.push(url.host_str().unwrap_or(""));
    values.push(url.path());
    values.push(url.query().unwrap_or(""));
    values.push(url.fragment().unwrap_or(""));
    values.push(url.domain().unwrap_or(""));

    Ok(csv::ByteRecord::from(values))
}

fn split_urls() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(io::stdin());
    let mut row = csv::StringRecord::new();

    let mut writer = csv::Writer::from_writer(io::stdout());

    while rdr.read_record(&mut row)? {
        let idx = row.get(0).expect("Must have an index!");
        if let Some(url) = row.get(1) {
            let output_row = match Url::parse(url) {
                Ok(url) => url_row(idx, url)?,
                Err(_) => empty_row(idx)?,
            };
            writer.write_record(&output_row)?;
        }
    }
    Ok(())
}
