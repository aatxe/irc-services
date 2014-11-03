extern crate irc;

use std::io::IoResult;
use std::io::fs::walk_dir;
use data::channel::Channel;
use irc::server::Server;
use irc::server::utils::Wrapper;
use irc::data::kinds::{IrcReader, IrcWriter};

mod chanserv;
mod nickserv;

pub fn process<'a, T, U>(server: &'a Wrapper<'a, T, U>, source: &str, command: &str, args: &[&str]) -> IoResult<()>
              where T: IrcWriter, U: IrcReader {
    let user = source.find('!').map_or("", |i| source[..i]);
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        if chan.starts_with("#") { return Ok(()); }
        let tokens: Vec<_> = msg.split_str(" ").collect();
        let res = if args.len() > 1 && upper_case(tokens[0])[] == "NS" {
            let cmd: String = upper_case(tokens[1]);
            match cmd[] {
                "REGISTER" => nickserv::Register::new(server, user, tokens),
                "IDENTIFY" => nickserv::Identify::new(server, user, tokens),
                "GHOST"    => nickserv::Ghost::new(server, user,tokens),
                "RECLAIM"  => nickserv::Reclaim::new(server, user, tokens),
                _          => Err(format!("{} is not a valid command.", tokens[1])),
            }
        } else if args.len() > 1 && upper_case(tokens[0])[] == "CS" {
            let cmd: String = upper_case(tokens[1]);
            match cmd[] {
                "REGISTER" => chanserv::Register::new(server, user, tokens),
                "ADMIN"    => chanserv::Admin::new(server, user, tokens),
                "OPER"     => chanserv::Oper::new(server, user, tokens),
                "VOICE"    => chanserv::Voice::new(server, user, tokens),
                "MODE"     => chanserv::Mode::new(server, user, tokens),
                "DEADMIN"  => chanserv::DeAdmin::new(server, user, tokens),
                _          => Err(format!("{} is not a valid command.", tokens[1])),
            }
        } else {
            Err("Commands must be prefixed by CS or NS.".into_string())
        };
        if let Err(msg) = res {
            try!(server.send_privmsg(user, msg[]));
        } else {
            try!(res.unwrap().do_func())
        }
    } else if let ("NOTICE", ["AUTH", suffix]) = (command, args) {
        if suffix.starts_with("***") {
            try!(server.identify());
        }
    } else if let ("376", _) = (command, args) {
        try!(start_up(server));
    } else if let ("422", _) = (command, args) {
        try!(start_up(server));
    } else if let ("JOIN", [chan]) = (command, args){
        if let Ok(channel) = Channel::load(chan) {
            let mode = if channel.owner[] == user {
                "+qa"
            } else if channel.admins[].contains(&user.into_string()) {
                "+a"
            } else if channel.opers[].contains(&user.into_string()) {
                "+o"
            } else if channel.voice[].contains(&user.into_string()) {
                "+v"
            } else {
                ""
            };
            if mode.len() > 0 {
                try!(server.send_samode(chan, mode[], user[]));
            }
        }
    }
    Ok(())
}

pub trait Functionality {
    fn do_func(&self) -> IoResult<()>;
}

fn start_up<T, U>(server: &Wrapper<T, U>) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    try!(server.send_oper(server.config().nickname[],
                      server.config().options.get_copy(&format!("oper-pass"))[]));
    let mut chans: Vec<String> = Vec::new();
    for path in try!(walk_dir(&Path::new("data/chanserv/"))) {
        let path_str = path.as_str().unwrap();
        let chan = path_str.find('.').map_or(String::new(), |i| path_str[14..i].into_string());
        if chan[] != "" {
            chans.push(chan);
        }
    }
    let mut join_line = String::new();
    for chan in chans.iter() {
        if join_line.len() < 40 && join_line.len() > 0 {
            join_line.push_str(",");
            join_line.push_str(chan[]);
        } else if join_line.len() == 0 {
            join_line.push_str(chan[]);
        } else {
            try!(server.send_join(join_line[]));
            join_line = chan.clone();
        }
    }
    try!(server.send_join(join_line[]));
    for chan in chans.iter() {
        try!(server.send_samode(chan[], "+a", server.config().nickname[]));
        let ch = try!(Channel::load(chan[]));
        if ch.mode.len() != 0 {
            try!(server.send_samode(chan[], ch.mode[], ""));
        }
    }
    Ok(())
}

fn upper_case(string: &str) -> String {
    string.chars().map(|c| c.to_uppercase()).collect()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::io::{MemReader, MemWriter};
    use data::channel::Channel;
    use irc::data::Config;
    use irc::conn::Connection;
    use irc::server::{IrcServer, Server};
    use irc::server::utils::Wrapper;

    pub fn test_helper(input: &str) -> String {
        let server = IrcServer::from_connection(Config {
                owners: vec!["test".into_string()],
                nickname: "test".into_string(),
                username: "test".into_string(),
                realname: "test".into_string(),
                password: String::new(),
                server: "irc.fyrechat.net".into_string(),
                port: 6667,
                channels: vec!["#test".into_string(), "#test2".into_string()],
                options: {
                    let mut map = HashMap::new();
                    map.insert("oper-pass".into_string(), "test".into_string());
                    map
                }
            },
            Connection::new(MemWriter::new(), MemReader::new(input.as_bytes().to_vec())),
        ).unwrap();
        for message in server.iter() {
            println!("{}", message);
            let mut args = Vec::new();
            let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
            args.push_all(msg_args[]);
            if let Some(ref suffix) = message.suffix {
                args.push(suffix[])
            }
            let source = message.prefix.unwrap_or(String::new());
            super::process(&Wrapper::new(&server), source[], message.command[], args[]).unwrap();
        }
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
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
    fn owner_on_join() {
        let mut ch = Channel::new("#test11", "test", "test").unwrap();
        ch.admins.push("test".into_string());
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test JOIN :#test11\r\n");
        assert_eq!(data[], "SAMODE #test11 +qa test\r\n");
    }

    #[test]
    fn admin_on_join() {
        let mut ch = Channel::new("#test8", "test", "owner").unwrap();
        ch.admins.push("test".into_string());
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test JOIN :#test8\r\n");
        assert_eq!(data[], "SAMODE #test8 +a test\r\n");
    }

    #[test]
    fn oper_on_join() {
        let mut ch = Channel::new("#test9", "test", "owner").unwrap();
        ch.opers.push("test".into_string());
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test JOIN :#test9\r\n");
        assert_eq!(data[], "SAMODE #test9 +o test\r\n");
    }

    #[test]
    fn voice_on_join() {
        let mut ch = Channel::new("#test10", "test", "owner").unwrap();
        ch.voice.push("test".into_string());
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test JOIN :#test10\r\n");
        assert_eq!(data[], "SAMODE #test10 +v test\r\n");
    }

    #[test]
    fn upper_case() {
        assert_eq!(super::upper_case("identify")[], "IDENTIFY")
    }
}
