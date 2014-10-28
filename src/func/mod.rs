extern crate irc;

use std::io::IoResult;
use irc::Bot;
use irc::bot::IrcBot;
use irc::data::{IrcReader, IrcWriter};

mod nickserv;

pub fn process<T, U>(bot: &IrcBot<T, U>, source: &str, command: &str, args: &[&str]) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        if chan.starts_with("#") { return Ok(()); }
        let user = source.find('!').map_or("", |i| source[..i]);
        let tokens: Vec<_> = msg.split_str(" ").collect();
        let res = match tokens[0] {
            "REGISTER" => nickserv::Register::new(user, tokens),
            _ => Err(format!("{} is not a valid command.", tokens[0])),
        };
        let msg = if let Err(msg) = res {
            msg
        } else {
            res.unwrap().do_func()
        };
        try!(bot.send_privmsg(user, msg[]));
    }
    Ok(())
}

pub trait Functionality {
    fn do_func(&self) -> String;
}

#[cfg(test)]
mod test {
    use std::io::{MemReader, MemWriter};
    use irc::Bot;
    use irc::bot::IrcBot;
    use irc::conn::Connection;

    pub fn test_helper(input: &str) -> String {
        let mut bot = IrcBot::from_connection(
            Connection::new(MemWriter::new(), MemReader::new(input.as_bytes().to_vec())).unwrap(),
            |bot, source, command, args| {
                super::process(bot, source, command, args)
            }
        ).unwrap();
        bot.output().unwrap();
        String::from_utf8(bot.conn.writer().deref().get_ref().to_vec()).unwrap()
    }
}
