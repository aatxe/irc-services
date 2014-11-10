use std::io::IoResult;
#[cfg(not(test))] use crypto::sbuf::StdHeapAllocator;
#[cfg(not(test))] use crypto::sha3::{hash, Sha3_512};
#[cfg(not(test))] use serialize::hex::ToHex;

pub mod channel;
pub mod state;
pub mod user;

pub type BotResult<T> = Result<T, String>;

#[cfg(not(test))]
pub fn password_hash(password: &str) -> IoResult<String> {
    let mut data = [0u8, ..64];
    try!(hash::<StdHeapAllocator>(Sha3_512, password.as_bytes(), data));
    Ok(data.to_hex())
}

#[cfg(test)]
pub fn password_hash(password: &str) -> IoResult<String> {
    Ok(password.into_string())
}
