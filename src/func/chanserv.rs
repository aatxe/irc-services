use super::Functionality;
use std::io::IoResult;
use data::BotResult;
use data::channel::Channel;
use irc::server::Server;
use irc::server::utils::Wrapper;
use irc::data::kinds::{IrcReader, IrcWriter};

pub struct Register<'a, T, U> where T: IrcWriter, U: IrcReader {
    server: &'a Wrapper<'a, T, U>,
    owner: String,
    channel: String,
    password: String,
}

impl<'a, T, U> Register<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err("Syntax: CS REGISTER channel password".into_string())
        }
        Ok(box Register {
            server: server,
            owner: user.into_string(),
            channel: args[2].into_string(),
            password: args[3].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T, U> Functionality for Register<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&self) -> IoResult<()> {
        let chan = try!(Channel::new(self.channel[], self.password[], self.owner[]));
        let msg = if Channel::exists(self.channel[]) {
            format!("Channel {} is already registered!", chan.name)
        } else if chan.save().is_ok() {;
            try!(self.server.send_samode(self.channel[], "+r", ""));
            try!(self.server.send_samode(self.channel[], "+qa", self.owner[]));
            try!(self.server.send_join(self.channel[]));
            try!(self.server.send_samode(self.channel[], "+a", self.server.config().nickname[]));
            format!("Channel {} has been registered. Don't forget the password!", chan.name)
        } else {
            format!("Failed to register {} due to an I/O issue.", chan.name)
        };
        self.server.send_privmsg(self.owner[], msg[])
    }
}

pub struct Admin<'a, T, U> where T: IrcWriter, U: IrcReader {
    server: &'a Wrapper<'a, T, U>,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T, U> Admin<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS ADMIN user channel password".into_string())
        }
        Ok(box Admin {
            server: server,
            owner: user.into_string(),
            channel: args[3].into_string(),
            password: args[4].into_string(),
            target: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T, U> Functionality for Admin<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !Channel::exists(self.channel[]) {
            format!("Channel {} is not registered!", self.channel[])
        } else if let Ok(mut chan) = Channel::load(self.channel[]) {
            if try!(chan.is_password(self.password[])) {
                chan.admins.push(self.target.clone());
                try!(chan.save());
                try!(self.server.send_samode(self.channel[], "+a", self.target[]));
                format!("{} is now an admin.", self.target[])
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to admin {} due to an I/O issue.", self.target[])
        };
        self.server.send_privmsg(self.owner[], msg[])
    }
}

pub struct Oper<'a, T, U> where T: IrcWriter, U: IrcReader {
    server: &'a Wrapper<'a, T, U>,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T, U> Oper<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS OPER user channel password".into_string())
        }
        Ok(box Oper {
            server: server,
            owner: user.into_string(),
            channel: args[3].into_string(),
            password: args[4].into_string(),
            target: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T, U> Functionality for Oper<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !Channel::exists(self.channel[]) {
            format!("Channel {} is not registered!", self.channel[])
        } else if let Ok(mut chan) = Channel::load(self.channel[]) {
            if try!(chan.is_password(self.password[])) {
                chan.opers.push(self.target.clone());
                try!(chan.save());
                try!(self.server.send_samode(self.channel[], "+o", self.target[]));
                format!("{} is now an oper.", self.target[])
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to oper {} due to an I/O issue.", self.target[])
        };
        self.server.send_privmsg(self.owner[], msg[])
    }
}

pub struct Voice<'a, T, U> where T: IrcWriter, U: IrcReader {
    server: &'a Wrapper<'a, T, U>,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T, U> Voice<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS VOICE user channel password".into_string())
        }
        Ok(box Voice {
            server: server,
            owner: user.into_string(),
            channel: args[3].into_string(),
            password: args[4].into_string(),
            target: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T, U> Functionality for Voice<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !Channel::exists(self.channel[]) {
            format!("Channel {} is not registered!", self.channel[])
        } else if let Ok(mut chan) = Channel::load(self.channel[]) {
            if try!(chan.is_password(self.password[])) {
                chan.voice.push(self.target.clone());
                try!(chan.save());
                try!(self.server.send_samode(self.channel[], "+v", self.target[]));
                format!("{} is now voiced.", self.target[])
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to voice {} due to an I/O issue.", self.target[])
        };
        self.server.send_privmsg(self.owner[], msg[])
    }
}

pub struct Mode<'a, T, U> where T: IrcWriter, U: IrcReader {
    server: &'a Wrapper<'a, T, U>,
    owner: String,
    channel: String,
    password: String,
    mode: String,
}

impl<'a, T, U> Mode<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS MODE mode channel password".into_string())
        }
        Ok(box Mode {
            server: server,
            owner: user.into_string(),
            channel: args[3].into_string(),
            password: args[4].into_string(),
            mode: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T, U> Functionality for Mode<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !Channel::exists(self.channel[]) {
            format!("Channel {} is not registered!", self.channel[])
        } else if let Ok(mut chan) = Channel::load(self.channel[]) {
            if try!(chan.is_password(self.password[])) {
                chan.mode = self.mode.clone();
                try!(chan.save());
                try!(self.server.send_samode(self.channel[], self.mode[], ""));
                format!("Channel mode is now {}.", self.mode[])
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to set channel mode {} due to an I/O issue.", self.mode[])
        };
        self.server.send_privmsg(self.owner[], msg[])
    }
}

pub struct DeAdmin<'a, T, U> where T: IrcWriter, U: IrcReader {
    server: &'a Wrapper<'a, T, U>,
    owner: &'a str,
    channel: &'a str,
    password: &'a str,
    target: &'a str,
}

impl<'a, T, U> DeAdmin<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS DEADMIN user channel password".into_string())
        }
        Ok(box Admin {
            server: server,
            owner: user.into_string(),
            channel: args[3].into_string(),
            password: args[4].into_string(),
            target: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T, U> Functionality for DeAdmin<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !Channel::exists(self.channel[]) {
            format!("Channel {} is not registered!", self.channel[])
        } else if let Ok(mut chan) = Channel::load(self.channel[]) {
            if try!(chan.is_password(self.password[])) {
                chan.admins.retain(|u| u[] != self.target[]);
                try!(chan.save());
                try!(self.server.send_samode(self.channel[], "-a", self.target[]));
                format!("{} is no longer an admin.", self.target[])
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to de-admin {} due to an I/O issue.", self.target[])
        };
        self.server.send_privmsg(self.owner[], msg[])
    }
}

#[cfg(test)]
mod test {
    use std::io::fs::unlink;
    use data::channel::Channel;
    use func::test::test_helper;

    #[test]
    fn register_succeeded() {
        let _ = unlink(&Path::new("data/chanserv/#test4.json"));
        let data = test_helper(":test2!test@test PRIVMSG test :CS REGISTER #test4 test");
        let mut exp = "SAMODE #test4 +r\r\n".into_string();
        exp.push_str("SAMODE #test4 +qa test2\r\n");
        exp.push_str("JOIN #test4\r\n");
        exp.push_str("SAMODE #test4 +a test\r\n");
        exp.push_str("PRIVMSG test2 :Channel #test4 has been registered. ");
        exp.push_str("Don't forget the password!\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn register_failed_channel_exists() {
        let ch = Channel::new("#test", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS REGISTER #test test");
        assert_eq!(data[], "PRIVMSG test :Channel #test is already registered!\r\n");
    }

    #[test]
    fn admin_succeeded() {
        let ch = Channel::new("#test5", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS ADMIN test2 #test5 test");
        assert_eq!(Channel::load("#test5").unwrap().admins, vec!("test2".into_string()));
        let mut exp = "SAMODE #test5 +a test2\r\n".into_string();
        exp.push_str("PRIVMSG test :test2 is now an admin.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn admin_failed_channel_unregistered() {
        let data = test_helper(":test!test@test PRIVMSG test :CS ADMIN test2 #unregistered test");
        assert_eq!(data[], "PRIVMSG test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn admin_failed_password_incorrect() {
        let ch = Channel::new("#test12", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS ADMIN test2 #test12 wrong");
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn oper_succeeded() {
        let ch = Channel::new("#test6", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS OPER test2 #test6 test");
        assert_eq!(Channel::load("#test6").unwrap().opers, vec!("test2".into_string()));
        let mut exp = "SAMODE #test6 +o test2\r\n".into_string();
        exp.push_str("PRIVMSG test :test2 is now an oper.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn oper_failed_channel_unregistered() {
        let data = test_helper(":test!test@test PRIVMSG test :CS OPER test2 #unregistered test");
        assert_eq!(data[], "PRIVMSG test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn oper_failed_password_incorrect() {
        let ch = Channel::new("#test13", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS OPER test2 #test13 wrong");
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn voice_succeeded() {
        let ch = Channel::new("#test7", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS VOICE test2 #test7 test");
        assert_eq!(Channel::load("#test7").unwrap().voice, vec!("test2".into_string()));
        let mut exp = "SAMODE #test7 +v test2\r\n".into_string();
        exp.push_str("PRIVMSG test :test2 is now voiced.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn voice_failed_channel_unregistered() {
        let data = test_helper(":test!test@test PRIVMSG test :CS VOICE test2 #unregistered test");
        assert_eq!(data[], "PRIVMSG test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn voice_failed_password_incorrect() {
        let ch = Channel::new("#test14", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS VOICE test2 #test14 wrong");
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn mode_succeeded() {
        let ch = Channel::new("#test15", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS MODE +i #test15 test");
        let mut exp = "SAMODE #test15 +i\r\n".into_string();
        exp.push_str("PRIVMSG test :Channel mode is now +i.\r\n");
        assert_eq!(data[], exp[])
    }

    #[test]
    fn mode_failed_channel_unregistered() {
        let data = test_helper(":test!test@test PRIVMSG test :CS MODE +i #unregistered test");
        assert_eq!(data[], "PRIVMSG test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn mode_failed_password_incorrect() {
        let ch = Channel::new("#test16", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS MODE +i #test16 wrong");
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }
}
