use std::io::Result as IoResult;
use std::io::Write;
use openssl::crypto::hash::{Type, Hasher};
use rustc_serialize::hex::ToHex;

pub mod channel;
#[cfg(feature = "democracy")] pub mod democracy;
#[cfg(feature = "derp")] pub mod derp;
#[cfg(feature = "resistance")] pub mod resistance;
pub mod state;
pub mod user;

pub type BotResult<T> = Result<T, String>;

pub fn password_hash(password: &str) -> IoResult<String> {
    let mut hasher = Hasher::new(Type::SHA512);
    try!(hasher.write_all(password.as_bytes()));
    Ok(hasher.finish().to_hex())
}

#[cfg(test)]
mod test {
    #[test]
    fn password_hash() {
        assert_eq!(&super::password_hash("test").unwrap()[..], "ee26b0dd4af7e749aa1a8ee3c10ae9923f61\
        8980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8\
        ff");
    }
}
