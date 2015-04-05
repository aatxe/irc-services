use super::password_hash;
use std::borrow::ToOwned;
use std::fs::{File, create_dir_all};
use std::io::{Error, ErrorKind, Result};
use std::io::prelude::*;
use std::path::Path;
use rustc_serialize::json::{decode, encode};

#[derive(RustcEncodable, RustcDecodable, Debug, PartialEq)]
pub struct Channel {
    pub name: String,
    pub password: String,
    pub owner: String,
    pub admins: Vec<String>,
    pub opers: Vec<String>,
    pub voice: Vec<String>,
    pub topic: String,
    pub mode: String,
}

impl Channel {
    pub fn new(name: &str, password: &str, owner: &str) -> Result<Channel> {
        Ok(Channel {
            name: name.to_owned(),
            password: try!(password_hash(password)),
            owner: owner.to_owned(),
            admins: Vec::new(), opers: Vec::new(), voice: Vec::new(),
            topic: String::new(),
            mode: String::new(),
        })
    }

    pub fn is_password(&self, password: &str) -> Result<bool> {
        Ok(self.password == try!(password_hash(password)))
    }

    pub fn exists(name: &str) -> bool {
        Path::new(&format!("data/chanserv/{}.json", name)).exists()
    }

    pub fn load(name: &str) -> Result<Channel> {
        let path = format!("data/chanserv/{}.json", name);
        let mut file = try!(File::open(Path::new(&path)));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        decode(&data).map_err(|_| Error::new(
            ErrorKind::InvalidInput, "Failed to decode channel data."
        ))
    }

    pub fn save(&self) -> Result<()> {
        let mut path = "data/chanserv/".to_owned();
        let _ = create_dir_all(Path::new(&path));
        path.push_str(&self.name);
        path.push_str(".json");
        let mut f = try!(File::create(Path::new(&path)));
        try!(f.write_all(try!(encode(self).map_err(|_| Error::new(
            ErrorKind::InvalidInput, "Failed to decode channel data."
        ))).as_bytes()));
        f.flush()
    }
}

#[cfg(test)]
mod test {
    use super::super::password_hash;
    use super::Channel;
    use std::borrow::ToOwned;
    use std::fs::remove_file;
    use std::path::Path;

    #[test]
    fn new() {
        assert_eq!(Channel::new("#test", "test", "test").unwrap(), Channel {
            name: "#test".to_owned(),
            password: password_hash("test").unwrap(),
            owner: "test".to_owned(),
            admins: Vec::new(), opers: Vec::new(), voice: Vec::new(),
            topic: "".to_owned(),
            mode: "".to_owned(),
        });
    }

    #[test]
    fn exists() {
        let ch = Channel::new("#test2", "test", "test").unwrap();
        let _ = remove_file(Path::new("data/chanserv/#test2.json"));
        assert!(!Channel::exists("#test2"));
        ch.save().unwrap();
        assert!(Channel::exists("#test2"));
    }

    #[test]
    fn save() {
        let ch = Channel::new("#test", "test", "test").unwrap();
        ch.save().unwrap();
    }

    #[test]
    fn load() {
        let ch = Channel::new("#test3", "test", "test").unwrap();
        ch.save().unwrap();
        let ld = Channel::load("#test3");
        assert!(ld.is_ok());
        assert_eq!(ld.unwrap(), ch);
    }
}
