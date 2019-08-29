use std::error;
use std::ops::Deref;

use csv;
use lazy_static::lazy_static;
use tldextract::{TldExtractor, TldOption};
use url::Url;

type Error = Box<dyn error::Error + 'static>;

pub trait OptionDeref<T: Deref> {
    fn as_deref(&self) -> Option<&T::Target>;
}

impl<T: Deref> OptionDeref<T> for Option<T> {
    fn as_deref(&self) -> Option<&T::Target> {
        self.as_ref().map(Deref::deref)
    }
}

pub fn parse_url(url: &str) -> csv::StringRecord {
    urlsplit_record(url)
        .or_else(|e| error_record(url, e))
        .unwrap()
}

lazy_static! {
    static ref EXTRACTOR: TldExtractor = {
        let option = TldOption {
            cache_path: Some(".tld_cache".to_string()),
            private_domains: false,
            update_local: false,
            naive_mode: false,
        };
        TldExtractor::new(option)
    };
}

// Produce an error record, showing only the error message.
fn error_record(url: &str, error: Error) -> Result<csv::StringRecord, Error> {
    let mut record = csv::StringRecord::from(vec![url, "", "", "", "", "", "", "", "", "", ""]);
    record.push_field(&error.to_string());
    Ok(record)
}

pub fn header_record() -> csv::StringRecord {
    csv::StringRecord::from(vec![
        "url",
        "scheme",
        "host",
        "path",
        "query",
        "fragment",
        "hostname",
        "domain",
        "subdomain",
        "suffix",
        "registration",
        "error",
    ])
}

// Make a url record from a URL string, using both TLDextract and
// url parsing.
fn urlsplit_record(url: &str) -> Result<csv::StringRecord, Error> {
    let mut values: Vec<String> = Vec::with_capacity(12);
    values.push(url.to_string());

    {
        // URL Parsing, which will exit early if there is an
        // error, because if the parsing fails, then we almost
        // certianly don't want to attempt the TLD extractor.
        let parts = Url::parse(url)?;
        values.push(parts.scheme().to_string());
        values.push(parts.host_str().unwrap_or("").to_string());
        values.push(parts.path().to_string());
        values.push(parts.query().unwrap_or("").to_string());
        values.push(parts.fragment().unwrap_or("").to_string());
        values.push(parts.domain().unwrap_or("").to_string());
    }

    match EXTRACTOR.extract(&url) {
        Ok(tld) => {
            values.push(tld.domain.as_deref().unwrap_or("").to_string());
            values.push(tld.subdomain.as_deref().unwrap_or("").to_string());
            values.push(tld.suffix.as_deref().unwrap_or("").to_string());

            let registration = if tld.suffix.is_some() {
                format!(
                    "{}.{}",
                    tld.domain.as_deref().unwrap_or(""),
                    tld.suffix.as_deref().unwrap_or("")
                )
            } else {
                tld.domain.as_deref().unwrap_or("").to_string()
            };

            values.push(registration);
            values.push("".into())
        }
        Err(err) => {
            values.push("".into());
            values.push("".into());
            values.push("".into());
            values.push("".into());
            values.push(err.to_string())
        }
    }

    Ok(csv::StringRecord::from(values))
}
