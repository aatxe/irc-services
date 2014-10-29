use super::Functionality;
use std::io::IoResult;
use data::BotResult;
use data::user::User;
use irc::Bot;

pub struct Register<'a> {
    bot: &'a Bot + 'a,
    nickname: String,
    password: String,
    email: Option<String>,
}

impl<'a> Register<'a> {
    pub fn new(bot: &'a Bot, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 2 && args.len() != 3 {
            return Err("Syntax: REGISTER password [email]".into_string())
        }
        Ok(box Register {
            bot: bot,
            nickname: user.into_string(),
            password: args[1].into_string(),
            email: if args.len() == 3 {
                Some(args[2].into_string())
            } else {
                None
            }
        } as Box<Functionality>)
    }
}

impl<'a> Functionality for Register<'a> {
    fn do_func(&self) -> IoResult<()> {
        let user = User::new(self.nickname[], self.password[], self.email.as_ref().map(|s| s[]));
        let msg = if User::exists(self.nickname[]) {
            format!("Nickname {} is already registered!", user.nickname)
        } else if user.save().is_ok() {
            format!("Nickname {} has been registered. Don't forget your password!", user.nickname)
        } else {
            format!("Failed to register {} for an unknown reason.", user.nickname)
        };
        self.bot.send_privmsg(self.nickname[], msg[])
    }
}

pub struct Identify<'a> {
    bot: &'a Bot + 'a,
    nickname: String,
    password: String,
}

impl<'a> Identify<'a> {
    pub fn new(bot: &'a Bot, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 2 {
            return Err("Syntax: IDENTIFY password".into_string())
        }
        Ok(box Identify {
            bot: bot,
            nickname: user.into_string(),
            password: args[1].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a> Functionality for Identify<'a> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !User::exists(self.nickname[]) {
            "Your nick isn't registered."
        } else if let Ok(user) = User::load(self.nickname[]) {
            if user.is_password(self.password[]) {
                try!(self.bot.send_mode(self.nickname[], "+r"));
                "Password accepted - you are now recognized."
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to identify for an unknown reason."
        };
        self.bot.send_privmsg(self.nickname[], msg)
    }
}

#[cfg(test)]
mod test {
    use std::io::fs::unlink;
    use data::user::User;
    use func::test::test_helper;

    #[test]
    fn register_succeeded() {
        let _ = unlink(&Path::new("data/nickserv/test4.json"));
        let data = test_helper(":test4!test@test PRIVMSG test :REGISTER test");
        assert_eq!(data[], "PRIVMSG test4 :Nickname test4 has been registered. Don't forget your password!\r\n");
    }

    #[test]
    fn register_failed_user_exists() {
        let u = User::new("test", "test", None);
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :REGISTER test");
        assert_eq!(data[], "PRIVMSG test :Nickname test is already registered!\r\n");
    }

    #[test]
    fn identify_succeeded() {
        let u = User::new("test5", "test", None);
        assert!(u.save().is_ok());
        let data = test_helper(":test5!test@test PRIVMSG test :IDENTIFY test");
        assert_eq!(data[], "MODE test5 :+r\r\nPRIVMSG test5 :Password accepted - you are now recognized.\r\n");
    }

    #[test]
    fn identify_failed_password_incorrect() {
        let u = User::new("test", "test", None);
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :IDENTIFY tset");
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn identify_failed_nickname_unregistered() {
        let data = test_helper(":unregistered!test@test PRIVMSG test :IDENTIFY test");
        assert_eq!(data[], "PRIVMSG unregistered :Your nick isn't registered.\r\n");
    }
}
