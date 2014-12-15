use std::io::IoResult;
use openssl::crypto::hash::{HashType, Hasher};
use serialize::hex::ToHex;

pub mod channel;
#[cfg(feature = "democracy")] pub mod democracy;
#[cfg(feature = "derp")] pub mod derp;
#[cfg(feature = "resistance")] pub mod resistance;
pub mod state;
pub mod user;

pub type BotResult<T> = Result<T, String>;

pub fn password_hash(password: &str) -> IoResult<String> {
    let mut hasher = Hasher::new(HashType::SHA512);
    try!(hasher.write_str(password));
    Ok(hasher.finalize().to_hex())
}
