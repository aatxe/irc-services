use super::password_hash;
use std::io::fs::{File, PathExtensions, mkdir_recursive};
use std::io::{FilePermission, InvalidInput, IoError, IoResult};
use serialize::json::{decode, encode};

#[deriving(Encodable, Decodable, Show, PartialEq)]
pub struct Channel {
    pub name: String,
    pub password: String,
    pub owner: String,
    pub admins: Vec<String>,
    pub opers: Vec<String>,
    pub voice: Vec<String>,
    pub mode: String,
}

impl Channel {
    pub fn new(name: &str, password: &str, owner: &str) -> IoResult<Channel> {
        Ok(Channel {
            name: name.into_string(),
            password: try!(password_hash(password)),
            owner: owner.into_string(),
            admins: Vec::new(), opers: Vec::new(), voice: Vec::new(),
            mode: String::new(),
        })
    }

    pub fn is_password(&self, password: &str) -> IoResult<bool> {
        Ok(self.password == try!(password_hash(password)))
    }

    pub fn exists(name: &str) -> bool {
        Path::new(format!("data/chanserv/{}.json", name)[]).exists()
    }

    pub fn load(name: &str) -> IoResult<Channel> {
        let mut path = "data/chanserv/".into_string();
        path.push_str(name);
        path.push_str(".json");
        let mut file = try!(File::open(&Path::new(path[])));
        let data = try!(file.read_to_string());
        decode(data[]).map_err(|e| IoError {
            kind: InvalidInput,
            desc: "Decoder error",
            detail: Some(e.to_string()),
        })
    }

    pub fn save(&self) -> IoResult<()> {
        let mut path = "data/chanserv/".into_string();
        try!(mkdir_recursive(&Path::new(path[]), FilePermission::all()));
        path.push_str(self.name[]);
        path.push_str(".json");
        let mut f = File::create(&Path::new(path[]));
        f.write_str(encode(self)[])
    }
}

#[cfg(test)]
mod test {
    use super::super::password_hash;
    use super::Channel;
    use std::io::fs::unlink;

    #[test]
    fn new() {
        assert_eq!(Channel::new("#test", "test", "test").unwrap(), Channel {
            name: "#test".into_string(),
            password: password_hash("test").unwrap(),
            owner: "test".into_string(),
            admins: Vec::new(), opers: Vec::new(), voice: Vec::new(),
            mode: "".into_string(),
        });
    }

    #[test]
    fn exists() {
        let ch = Channel::new("#test2", "test", "test").unwrap();
        let _ = unlink(&Path::new("data/chanserv/#test2.json"));
        assert!(!Channel::exists("#test2"));
        assert!(ch.save().is_ok());
        assert!(Channel::exists("#test2"));
    }

    #[test]
    fn save() {
        let ch = Channel::new("#test", "test", "test").unwrap();
        assert!(ch.save().is_ok());
    }

    #[test]
    fn load() {
        let ch = Channel::new("#test3", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let ld = Channel::load("#test3");
        assert!(ld.is_ok());
        assert_eq!(ld.unwrap(), ch);
    }
}
