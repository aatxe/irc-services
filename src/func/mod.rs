extern crate irc;

use std::io::IoResult;
use irc::Bot;
use irc::bot::IrcBot;
use irc::data::{IrcReader, IrcWriter};

mod chanserv;
mod nickserv;

pub fn process<T, U>(bot: &IrcBot<T, U>, source: &str, command: &str, args: &[&str]) -> IoResult<()>
              where T: IrcWriter, U: IrcReader {
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        if chan.starts_with("#") { return Ok(()); }
        let user = source.find('!').map_or("", |i| source[..i]);
        let tokens: Vec<_> = msg.split_str(" ").collect();
        let res = if args.len() > 1 && upper_case(tokens[0])[] == "NS" {
            let cmd: String = upper_case(tokens[1]);
            match cmd[] {
                "REGISTER" => nickserv::Register::new(bot, user, tokens),
                "IDENTIFY" => nickserv::Identify::new(bot, user, tokens),
                "GHOST"    => nickserv::Ghost::new(bot, user,tokens),
                "RECLAIM"  => nickserv::Reclaim::new(bot, user, tokens),
                _ => Err(format!("{} is not a valid command.", tokens[1])),
            }
        } else if args.len() > 1 && upper_case(tokens[0])[] == "CS" {
            let cmd: String = upper_case(tokens[1]);
            match cmd[] {
                "REGISTER" => chanserv::Register::new(bot, user, tokens),
                _ => Err(format!("{} is not a valid command.", tokens[1])),
            }
        } else {
            Err("Commands must be prefixed by CS or NS.".into_string())
        };
        if let Err(msg) = res {
            try!(bot.send_privmsg(user, msg[]));
        } else {
            try!(res.unwrap().do_func())
        }
    } else if let ("376", _) = (command, args) {
        try!(bot.send_oper(bot.config().nickname[],
                      bot.config().options.get_copy(&format!("oper-pass"))[]));
    } else if let ("422", _) = (command, args) {
        try!(bot.send_oper(bot.config().nickname[],
                      bot.config().options.get_copy(&format!("oper-pass"))[]));
    }
    Ok(())
}

pub trait Functionality {
    fn do_func(&self) -> IoResult<()>;
}

fn upper_case(string: &str) -> String {
    string.chars().map(|c| c.to_uppercase()).collect()
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

    #[test]
    fn commands_must_be_prefxed() {
        let data = test_helper(":test!test@test PRIVMSG test :IDENTIFY\r\n");
        assert_eq!(data[], "PRIVMSG test :Commands must be prefixed by CS or NS.\r\n")
    }

    #[test]
    fn non_command_message_in_channel() {
        let data = test_helper(":test!test@test PRIVMSG #test :Hi there!\r\n");
        assert_eq!(data[], "");
    }

    #[test]
    fn non_command_message_in_query() {
        let data = test_helper(":test!test@test PRIVMSG test :CS line\r\n");
        assert_eq!(data[], "PRIVMSG test :line is not a valid command.\r\n");
    }

    #[test]
    fn upper_case() {
        assert_eq!(super::upper_case("identify")[], "IDENTIFY")
    }
}
