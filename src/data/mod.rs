use std::io::fs::{File, mkdir_recursive};
use std::io::{FilePermission, InvalidInput, IoError, IoResult};
use serialize::json::{decode, encode};

#[deriving(Encodable, Decodable, Show, PartialEq)]
pub struct User {
    pub nickname: String,
    pub password: String,
    pub email: Option<String>,
}

impl User {
    pub fn new(nickname: &str, password: &str, email: Option<&str>) -> User {
        User {
            nickname: nickname.into_string(),
            password: password.into_string(),
            email: if let Some(address) = email {
                Some(address.into_string())
            } else {
                None
            },
        }
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
        let _ = mkdir_recursive(&Path::new(path[]), FilePermission::all());
        path.push_str(self.nickname[]);
        path.push_str(".json");
        let mut f = File::create(&Path::new(path[]));
        f.write_str(encode(self)[])
    }
}

#[cfg(test)]
mod test {
    use super::User;

    #[test]
    fn new() {
        assert_eq!(User::new("test", "test", None), User {
            nickname: "test".into_string(),
            password: "test".into_string(),
            email: None,
        });
        assert_eq!(User::new("test", "test", Some("test@test.com")), User {
            nickname: "test".into_string(),
            password: "test".into_string(),
            email: Some("test@test.com".into_string()),
        });
    }

    #[test]
    fn save() {
        let u = User::new("test", "test", None);
        assert!(u.save().is_ok());
    }

    #[test]
    fn load() {
        let u = User::new("test2", "test", None);
        assert!(u.save().is_ok());
        let v = User::load("test2");
        assert!(v.is_ok());
        assert_eq!(v.unwrap(), User {
            nickname: "test2".into_string(),
            password: "test".into_string(),
            email: None,
        });
    }
}
