#![feature(box_syntax, collections, fs_walk, io, path_ext)]
#![cfg_attr(feature = "resistance", feature(core))]
extern crate irc;
extern crate openssl;
#[cfg(feature = "resistance")] extern crate rand;
extern crate rustc_serialize;

#[cfg(not(test))] use data::state::State;
#[cfg(not(test))] use irc::client::prelude::*;

mod data;
mod func;

#[cfg(not(test))]
fn main() {
    let server = IrcServer::new("config.json").unwrap();
    let state = State::new();
    for message in server.iter() {
        let message = message.unwrap();
        print!("{}", message.into_string());
        let mut args = Vec::new();
        let msg_args: Vec<_> = message.args.iter().map(|s| &s[..]).collect();
        args.push_all(&msg_args);
        if let Some(ref suffix) = message.suffix {
            args.push(&suffix)
        }
        let source = message.get_source_nickname().unwrap_or("");
        func::process(&server, source, &message.command, &args, &state).unwrap();
    }
}
