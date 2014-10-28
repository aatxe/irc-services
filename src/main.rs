extern crate irc;

use irc::Bot;
use irc::bot::IrcBot;

fn main() {
    let mut bot = IrcBot::new(|_, _, _, _| {
        Ok(())
    }).unwrap();
    bot.identify().unwrap();
    bot.output().unwrap();
}
