use super::Functionality;
use std::io::IoResult;
use data::BotResult;
use data::channel::Channel;
use irc::Bot;

pub struct Register<'a> {
    bot: &'a Bot + 'a,
    owner: String,
    channel: String,
    password: String,
}

impl<'a> Register<'a> {
    pub fn new(bot: &'a Bot, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err("Syntax: CS REGISTER channel password".into_string())
        }
        Ok(box Register {
            bot: bot,
            owner: user.into_string(),
            channel: args[2].into_string(),
            password: args[3].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a> Functionality for Register<'a> {
    fn do_func(&self) -> IoResult<()> {
        let chan = try!(
            Channel::new(self.channel[], self.password[], self.owner[])
        );
        let msg = if Channel::exists(self.channel[]) {
            format!("Channel {} is already registered!", chan.name)
        } else if chan.save().is_ok() {;
            try!(self.bot.send_samode(self.channel[], "+r"));
            try!(self.bot.send_samode(self.channel[], format!("+qa {}", self.owner)[]));
            try!(self.bot.send_join(self.channel[]));
            try!(self.bot.send_samode(self.channel[], format!("+a {}", self.bot.config().nickname)[]));
            format!("Channel {} has been registered. Don't forget the password!", chan.name)
        } else {
            format!("Failed to register {} due to an I/O issue.", chan.name)
        };
        self.bot.send_privmsg(self.owner[], msg[])
    }
}

pub struct Admin<'a> {
    bot: &'a Bot + 'a,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a> Admin<'a> {
    pub fn new(bot: &'a Bot, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS ADMIN user channel password".into_string())
        }
        Ok(box Admin {
            bot: bot,
            owner: user.into_string(),
            channel: args[3].into_string(),
            password: args[4].into_string(),
            target: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a> Functionality for Admin<'a> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !Channel::exists(self.channel[]) {
            format!("Channel {} is not registered!", self.channel[])
        } else if let Ok(mut chan) = Channel::load(self.channel[]) {
            if try!(chan.is_password(self.password[])) {
                chan.admins.push(self.target.clone());
                try!(chan.save());
                try!(self.bot.send_samode(self.channel[], format!("+a {}", self.target[])[]));
                format!("{} is now an admin.", self.target[])
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to admin {} due to an I/O issue.", self.target[])
        };
        self.bot.send_privmsg(self.owner[], msg[])
    }
}

pub struct Oper<'a> {
    bot: &'a Bot + 'a,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a> Oper<'a> {
    pub fn new(bot: &'a Bot, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS OPER user channel password".into_string())
        }
        Ok(box Oper {
            bot: bot,
            owner: user.into_string(),
            channel: args[3].into_string(),
            password: args[4].into_string(),
            target: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a> Functionality for Oper<'a> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !Channel::exists(self.channel[]) {
            format!("Channel {} is not registered!", self.channel[])
        } else if let Ok(mut chan) = Channel::load(self.channel[]) {
            if try!(chan.is_password(self.password[])) {
                chan.opers.push(self.target.clone());
                try!(chan.save());
                try!(self.bot.send_samode(self.channel[], format!("+o {}", self.target[])[]));
                format!("{} is now an oper.", self.target[])
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to oper {} due to an I/O issue.", self.target[])
        };
        self.bot.send_privmsg(self.owner[], msg[])
    }
}

pub struct Voice<'a> {
    bot: &'a Bot + 'a,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a> Voice<'a> {
    pub fn new(bot: &'a Bot, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS VOICE user channel password".into_string())
        }
        Ok(box Voice {
            bot: bot,
            owner: user.into_string(),
            channel: args[3].into_string(),
            password: args[4].into_string(),
            target: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a> Functionality for Voice<'a> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !Channel::exists(self.channel[]) {
            format!("Channel {} is not registered!", self.channel[])
        } else if let Ok(mut chan) = Channel::load(self.channel[]) {
            if try!(chan.is_password(self.password[])) {
                chan.voice.push(self.target.clone());
                try!(chan.save());
                try!(self.bot.send_samode(self.channel[], format!("+v {}", self.target[])[]));
                format!("{} is now voiced.", self.target[])
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to voice {} due to an I/O issue.", self.target[])
        };
        self.bot.send_privmsg(self.owner[], msg[])
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
        exp.push_str("JOIN :#test4\r\n");
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
}
