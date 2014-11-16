#![cfg(feature = "derp")]
use std::io::{File, FilePermission, InvalidInput, IoError, IoResult};
use std::io::fs::mkdir_recursive;
use serialize::json::{decode, encode};

#[deriving(Encodable, Decodable, Show, PartialEq)]
pub struct DerpCounter {
    derps: uint,
}

impl DerpCounter {
    pub fn load() -> IoResult<DerpCounter> {
        let path = "data/derp.json".into_string();
        let file = File::open(&Path::new(path[]));
        if let Ok(mut file) = file {
            let data = try!(file.read_to_string());
            decode(data[]).map_err(|e| IoError {
                kind: InvalidInput,
                desc: "Decoder error",
                detail: Some(e.to_string()),
            })
        } else {
            Ok(DerpCounter { derps: 0 })
        }
    }

    pub fn save(&self) -> IoResult<()> {
        let mut path = "data/".into_string();
        try!(mkdir_recursive(&Path::new(path[]), FilePermission::all()));
        path.push_str("derp.json");
        let mut f = File::create(&Path::new(path[]));
        f.write_str(encode(self)[])
    }

    pub fn increment(&mut self) {
        self.derps += 1
    }

    pub fn derps(&self) -> uint {
        self.derps
    }
}

#[cfg(test)]
mod test {
    use super::DerpCounter;
    
    #[test]
    fn save() {
        let derp = DerpCounter::load();
        assert!(derp.is_ok())
        let mut derp = derp.unwrap();
        derp.increment();
        assert!(derp.save().is_ok());
    }
}
