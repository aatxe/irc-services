use super::password_hash;
use std::borrow::ToOwned;
use std::error::Error as StdError;
use std::fs::{File, create_dir_all};
use std::io::{Error, ErrorKind, Result};
use std::io::prelude::*;
use std::path::Path;
use rustc_serialize::json::{decode, encode};

#[derive(RustcEncodable, RustcDecodable, Debug, PartialEq)]
pub struct User {
    pub nickname: String,
    pub password: String,
    pub email: Option<String>,
}

impl User {
    pub fn new(nickname: &str, password: &str, email: Option<&str>) -> Result<User> {
        Ok(User {
            nickname: nickname.to_owned(),
            password: try!(password_hash(password)),
            email: email.map(|s| s.to_owned()),
        })
    }

    pub fn update_password(&mut self, password: &str) -> Result<()> {
        self.password = try!(password_hash(password));
        Ok(())
    }

    pub fn is_password(&self, password: &str) -> Result<bool> {
        Ok(self.password == try!(password_hash(password)))
    }

    pub fn exists(nickname: &str) -> bool {
        Path::new(&format!("data/nickserv/{}.json", nickname)).exists()
    }

    pub fn load(nickname: &str) -> Result<User> {
        let path = format!("data/nickserv/{}.json", nickname);
        let mut file = try!(File::open(Path::new(&path)));
        let mut data = String::new();
        try!(file.read_to_string(&mut data));
        decode(&data).map_err(|e| 
            Error::new(ErrorKind::InvalidInput, "Failed to decode user data.",
                       Some(e.description().to_owned()))
        )
    }

    pub fn save(&self) -> Result<()> {
        let mut path = "data/nickserv/".to_owned();
        let _ = create_dir_all(Path::new(&path));
        path.push_str(&self.nickname);
        path.push_str(".json");
        let mut f = try!(File::create(Path::new(&path)));
        try!(f.write_all(try!(encode(self).map_err(|e| 
            Error::new(ErrorKind::InvalidInput, "Failed to decode channel data.",
                       Some(e.description().to_owned()))
        )).as_bytes()));
        f.flush()
    }
}

#[cfg(test)]
mod test {
    use super::super::password_hash;
    use super::User;
    use std::borrow::ToOwned;
    use std::fs::remove_file;
    use std::path::Path;

    #[test]
    fn new() {
        assert_eq!(User::new("test", "test", None).unwrap(), User {
            nickname: "test".to_owned(),
            password: password_hash("test").unwrap(),
            email: None,
        });
        assert_eq!(User::new("test", "test", Some("test@test.com")).unwrap(), User {
            nickname: "test".to_owned(),
            password: password_hash("test").unwrap(),
            email: Some("test@test.com".to_owned()),
        });
    }

    #[test]
    fn exists() {
        let u = User::new("test2", "test", None).unwrap();
        let _ = remove_file(Path::new("data/nickserv/test2.json"));
        assert!(!User::exists("test2"));
        u.save().unwrap();
        assert!(User::exists("test2"));
    }

    #[test]
    fn save() {
        let u = User::new("test", "test", None).unwrap();
        u.save().unwrap();
    }

    #[test]
    fn load() {
        let u = User::new("test3", "test", None).unwrap();
        u.save().unwrap();
        let v = User::load("test3");
        assert!(v.is_ok());
        assert_eq!(v.unwrap(), u);
    }
}
