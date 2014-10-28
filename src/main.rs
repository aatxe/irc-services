#![feature(if_let)]
#![feature(slicing_syntax)]
extern crate irc;
extern crate serialize;

use irc::Bot;
use irc::bot::IrcBot;

mod data;

fn main() {
    let mut bot = IrcBot::new(|_, _, _, _| {
        Ok(())
    }).unwrap();
    bot.identify().unwrap();
    bot.output().unwrap();
}
