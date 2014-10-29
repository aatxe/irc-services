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
            format!("Failed to register {} for an unknown reason.", chan.name)
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
    fn register_failed_user_exists() {
        let ch = Channel::new("#test", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :CS REGISTER #test test");
        assert_eq!(data[], "PRIVMSG test :Channel #test is already registered!\r\n");
    }
}
