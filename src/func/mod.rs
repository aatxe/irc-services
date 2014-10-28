extern crate irc;

use std::io::IoResult;
use data::BotResult;
use irc::Bot;
use irc::bot::IrcBot;
use irc::data::{IrcReader, IrcWriter};

mod nickserv;

pub fn process<T, U>(bot: &IrcBot<T, U>, source: &str, command: &str, args: &[&str]) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        if chan.starts_with("#") { return Ok(()); }
        let user = source.find('!').map_or("", |i| source[..i]);
        let tokens: Vec<_> = msg.split_str(" ").collect();
        match tokens[0] {
            "register" => (),
            "identify" => (),
            _ => (),
        }
    }
    Ok(())
}

pub trait Functionality {
    fn do_func(&mut self) -> BotResult<()>;
}
