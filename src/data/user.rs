use std::io::fs::{File, PathExtensions, mkdir_recursive};
use std::io::{FilePermission, InvalidInput, IoError, IoResult};
use crypto::sbuf::StdHeapAllocator;
use crypto::sha3::{hash, Sha3_512};
use serialize::hex::ToHex;
use serialize::json::{decode, encode};

#[deriving(Encodable, Decodable, Show, PartialEq)]
pub struct User {
    pub nickname: String,
    pub password: String,
    pub email: Option<String>,
}

impl User {
    pub fn new(nickname: &str, password: &str, email: Option<&str>) -> IoResult<User> {
        Ok(User {
            nickname: nickname.into_string(),
            password: try!(User::password_hash(password)),
            email: email.map(|s| s.into_string()),
        })
    }

    pub fn is_password(&self, password: &str) -> IoResult<bool> {
        Ok(self.password == try!(User::password_hash(password)))
    }

    pub fn exists(nickname: &str) -> bool {
        Path::new(format!("data/nickserv/{}.json", nickname)[]).exists()
    }

    pub fn load(nickname: &str) -> IoResult<User> {
        let mut path = "data/nickserv/".into_string();
        path.push_str(nickname);
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
        let mut path = "data/nickserv/".into_string();
        try!(mkdir_recursive(&Path::new(path[]), FilePermission::all()));
        path.push_str(self.nickname[]);
        path.push_str(".json");
        let mut f = File::create(&Path::new(path[]));
        f.write_str(encode(self)[])
    }

    pub fn password_hash(password: &str) -> IoResult<String> {
        let mut data = [0u8, ..64];
        try!(hash::<StdHeapAllocator>(Sha3_512, password.as_bytes(), data));
        Ok(data.to_hex())
    }
}

#[cfg(test)]
mod test {
    use super::User;
    use std::io::fs::unlink;

    #[test]
    fn new() {
        assert_eq!(User::new("test", "test", None).unwrap(), User {
            nickname: "test".into_string(),
            password: User::password_hash("test").unwrap(),
            email: None,
        });
        assert_eq!(User::new("test", "test", Some("test@test.com")).unwrap(), User {
            nickname: "test".into_string(),
            password: User::password_hash("test").unwrap(),
            email: Some("test@test.com".into_string()),
        });
    }

    #[test]
    fn exists() {
        let u = User::new("test2", "test", None).unwrap();
        let _ = unlink(&Path::new("data/nickserv/test2.json"));
        assert!(!User::exists("test2"));
        assert!(u.save().is_ok());
        assert!(User::exists("test2"));
    }

    #[test]
    fn save() {
        let u = User::new("test", "test", None).unwrap();
        assert!(u.save().is_ok());
    }

    #[test]
    fn load() {
        let u = User::new("test3", "test", None).unwrap();
        assert!(u.save().is_ok());
        let v = User::load("test3");
        assert!(v.is_ok());
        assert_eq!(v.unwrap(), User {
            nickname: "test3".into_string(),
            password: User::password_hash("test").unwrap(),
            email: None,
        });
    }
}
