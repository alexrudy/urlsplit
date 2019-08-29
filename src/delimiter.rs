use serde::de::{Deserialize, Deserializer, Error};

#[derive(Debug, Clone, Copy)]
pub struct Delimiter(pub u8);

impl<'de> Deserialize<'de> for Delimiter {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Delimiter, D::Error> {
        let c = String::deserialize(d)?;
        match &*c {
            r"\t" => Ok(Delimiter(b'\t')),
            s => {
                if s.len() != 1 {
                    let msg = format!(
                        "Could not convert '{}' to a single \
                         ASCII character.",
                        s
                    );
                    return Err(D::Error::custom(msg));
                }
                let c = s.chars().next().unwrap();
                if c.is_ascii() {
                    Ok(Delimiter(c as u8))
                } else {
                    let msg = format!(
                        "Could not convert '{}' \
                         to ASCII delimiter.",
                        c
                    );
                    Err(D::Error::custom(msg))
                }
            }
        }
    }
}
