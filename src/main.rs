#![feature(if_let)]
#![feature(slicing_syntax)]
extern crate irc;
extern crate serialize;

use irc::Bot;
use irc::bot::IrcBot;

mod data;
mod func;

#[cfg(not(test))]
fn main() {
    let mut bot = IrcBot::new(func::process).unwrap();
    bot.identify().unwrap();
    bot.send_oper(bot.config().nickname[],
                  bot.config().options.get_copy(&format!("oper-pass"))[]).unwrap();
    bot.output().unwrap();
}
