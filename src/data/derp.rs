#![cfg(feature = "derp")]
use std::borrow::ToOwned;
use std::fs::{File, create_dir_all};
use std::io::{Error, ErrorKind, Result};
use std::io::prelude::*;
use std::path::Path;
use rustc_serialize::json::{decode, encode};

#[derive(RustcEncodable, RustcDecodable, Debug, PartialEq)]
pub struct DerpCounter {
    derps: usize,
}

impl DerpCounter {
    pub fn load() -> Result<DerpCounter> {
        let path = "data/derp.json".to_owned();
        let file = File::open(&Path::new(&path));
        if let Ok(mut file) = file {
            let mut data = String::new();
            try!(file.read_to_string(&mut data));
            decode(&data).map_err(|_| Error::new(
                ErrorKind::InvalidInput, "Failed to decode derp data."
            ))
        } else {
            Ok(DerpCounter { derps: 0 })
        }
    }

    pub fn save(&self) -> Result<()> {
        let mut path = "data/".to_owned();
        try!(create_dir_all(Path::new(&path)));
        path.push_str("derp.json");
        let mut f = try!(File::create(Path::new(&path)));
        try!(f.write_all(try!(encode(self).map_err(|_| Error::new(
            ErrorKind::InvalidInput, "Failed to encode derp data."
        ))).as_bytes()));
        f.flush()
    }

    pub fn increment(&mut self) {
        self.derps += 1
    }

    pub fn derps(&self) -> usize {
        self.derps
    }
}

#[cfg(test)]
mod test {
    use super::DerpCounter;
    
    #[test]
    fn save() {
        let derp = DerpCounter::load();
        assert!(derp.is_ok());
        let mut derp = derp.unwrap();
        derp.increment();
        assert!(derp.save().is_ok());
    }
}
