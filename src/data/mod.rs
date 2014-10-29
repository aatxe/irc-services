use std::io::IoResult;
use crypto::sbuf::StdHeapAllocator;
use crypto::sha3::{hash, Sha3_512};
use serialize::hex::ToHex;

pub mod channel;
pub mod user;

pub type BotResult<T> = Result<T, String>;

pub fn password_hash(password: &str) -> IoResult<String> {
    let mut data = [0u8, ..64];
    try!(hash::<StdHeapAllocator>(Sha3_512, password.as_bytes(), data));
    Ok(data.to_hex())
}
