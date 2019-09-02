use std::error;
use std::ops::Deref;

use csv;
use lazy_static::lazy_static;
use tldextract::{TldExtractor, TldOption};
use url::{self, Url};

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
fn error_record<E: error::Error>(url: &str, error: E) -> Result<csv::StringRecord, E> {
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

fn urlsplit_tld(url: &str, values: &mut Vec<String>) -> Result<(), url::ParseError> {
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
    Ok(())
}

// URL Parsing, which will exit early if there is an
// error, because if the parsing fails, then we almost
// certianly don't want to attempt the TLD extractor.
fn urlsplit_parse(url: &str, values: &mut Vec<String>) -> Result<(), url::ParseError> {
    let parts = Url::parse(url)?;
    values.push(parts.scheme().to_string());
    values.push(parts.host_str().unwrap_or("").to_string());
    values.push(parts.path().to_string());
    values.push(parts.query().unwrap_or("").to_string());
    values.push(parts.fragment().unwrap_or("").to_string());
    values.push(parts.domain().unwrap_or("").to_string());

    Ok(())
}

// Make a url record from a URL string, using both TLDextract and
// url parsing.
fn urlsplit_record(url: &str) -> Result<csv::StringRecord, url::ParseError> {
    let mut values: Vec<String> = Vec::with_capacity(12);
    values.push(url.to_string());
    urlsplit_parse(url, &mut values)?;
    urlsplit_tld(url, &mut values)?;
    Ok(csv::StringRecord::from(values))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::cmp;

    fn v<F, E>(urlfunc: F, url: &str) -> Result<Vec<String>, E>
    where
        F: Fn(&str, &mut Vec<String>) -> Result<(), E>,
        E: cmp::PartialEq,
    {
        let mut values = Vec::new();
        match urlfunc(url, &mut values) {
            Ok(()) => Ok(values),
            Err(e) => Err(e),
        }
    }

    #[test]
    fn test_urlsplit_parse() {
        assert_eq!(
            v(urlsplit_parse, "foo"),
            Err(url::ParseError::RelativeUrlWithoutBase)
        );
        assert_eq!(
            v(urlsplit_parse, "https://foo"),
            Ok(vec![
                "https".into(),
                "foo".into(),
                "/".into(),
                "".into(),
                "".into(),
                "foo".into()
            ])
        );

        assert_eq!(
            v(
                urlsplit_parse,
                "https://my.example.com/path/to/resource?query=hello#fragment"
            ),
            Ok(vec![
                "https".into(),
                "my.example.com".into(),
                "/path/to/resource".into(),
                "query=hello".into(),
                "fragment".into(),
                "my.example.com".into()
            ])
        );
    }

}
