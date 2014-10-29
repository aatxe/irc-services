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
        let user = try!(
            User::new(self.nickname[], self.password[], self.email.as_ref().map(|s| s[]))
        );
        let msg = if User::exists(self.nickname[]) {
            format!("Nickname {} is already registered!", user.nickname)
        } else if user.save().is_ok() {;
            try!(self.bot.send_samode(self.nickname[], "+r"));
            format!("Nickname {} has been registered. Don't forget your password!\r\n{}",
                    user.nickname, "You're now identified.")
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
            if try!(user.is_password(self.password[])) {
                try!(self.bot.send_samode(self.nickname[], "+r"));
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

pub struct Ghost<'a> {
    bot: &'a Bot + 'a,
    current_nick: String,
    nickname: String,
    password: String,
}

impl<'a> Ghost<'a> {
    pub fn new(bot: &'a Bot, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 {
            return Err("Syntax: GHOST nickname password".into_string())
        }
        Ok(box Ghost {
            bot: bot,
            current_nick: user.into_string(),
            nickname: args[1].into_string(),
            password: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a> Functionality for Ghost<'a> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !User::exists(self.nickname[]) {
            "That nick isn't registered, and therefore cannot be ghosted."
        } else if let Ok(user) = User::load(self.nickname[]) {
            if try!(user.is_password(self.password[])) {
                try!(self.bot.send_kill(self.nickname[],
                     format!("Ghosted by {}.", self.current_nick)[]));
                try!(self.bot.send_privmsg(self.nickname[], "User has been ghosted."));
                return Ok(());
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to ghost nick for an unknown reason."
        };
        self.bot.send_privmsg(self.current_nick[], msg)
    }
}

pub struct Reclaim<'a> {
    bot: &'a Bot + 'a,
    current_nick: String,
    nickname: String,
    password: String,
}

impl<'a> Reclaim<'a> {
    pub fn new(bot: &'a Bot, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 {
            return Err("Syntax: RECLAIM nickname password".into_string())
        }
        Ok(box Reclaim {
            bot: bot,
            current_nick: user.into_string(),
            nickname: args[1].into_string(),
            password: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a> Functionality for Reclaim<'a> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !User::exists(self.nickname[]) {
            "That nick isn't registered, and therefore cannot be reclaimed."
        } else if let Ok(user) = User::load(self.nickname[]) {
            if try!(user.is_password(self.password[])) {
                try!(self.bot.send_kill(self.nickname[],
                     format!("Reclaimed by {}.", self.current_nick)[]));
                try!(self.bot.send_sanick(self.current_nick[], self.nickname[]));
                try!(self.bot.send_samode(self.nickname[], "+r"));
                try!(self.bot.send_privmsg(self.nickname[],
                                           "Password accepted - you are now recognized."));
                return Ok(());
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to reclaim nick for an unknown reason."
        };
        self.bot.send_privmsg(self.current_nick[], msg)
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
        let mut exp = "SAMODE test4 :+r\r\n".into_string();
        exp.push_str("PRIVMSG test4 :Nickname test4 has been registered. ");
        exp.push_str("Don't forget your password!\r\n");
        exp.push_str("PRIVMSG test4 :You're now identified.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn register_failed_user_exists() {
        let u = User::new("test", "test", None).unwrap();
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :REGISTER test");
        assert_eq!(data[], "PRIVMSG test :Nickname test is already registered!\r\n");
    }

    #[test]
    fn identify_succeeded() {
        let u = User::new("test5", "test", None).unwrap();
        assert!(u.save().is_ok());
        let data = test_helper(":test5!test@test PRIVMSG test :IDENTIFY test");
        let mut exp = "SAMODE test5 :+r\r\n".into_string();
        exp.push_str("PRIVMSG test5 :Password accepted - you are now recognized.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn identify_failed_password_incorrect() {
        let u = User::new("test", "test", None).unwrap();
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :IDENTIFY tset");
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn identify_failed_nickname_unregistered() {
        let data = test_helper(":unregistered!test@test PRIVMSG test :IDENTIFY test");
        assert_eq!(data[], "PRIVMSG unregistered :Your nick isn't registered.\r\n");
    }

    #[test]
    fn ghost_succeeded() {
        let u = User::new("test6", "test", None).unwrap();
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :GHOST test6 test");
        let mut exp = "KILL test6 :Ghosted by test.\r\n".into_string();
        exp.push_str("PRIVMSG test6 :User has been ghosted.\r\n");
        assert_eq!(data[], exp[]);
    }


    #[test]
    fn ghost_failed_password_incorrect() {
        let u = User::new("test6", "test", None).unwrap();
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :GHOST test6 tset");
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn ghost_failed_nickname_unregistered() {
        let data = test_helper(":test!test@test PRIVMSG test :GHOST unregistered test");
        let mut exp = "PRIVMSG test :That nick isn't registered, ".into_string();
        exp.push_str("and therefore cannot be ghosted.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn reclaim_succeeded() {
        let u = User::new("test6", "test", None).unwrap();
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :RECLAIM test6 test");
        let mut exp = "KILL test6 :Reclaimed by test.\r\n".into_string();
        exp.push_str("SANICK test :test6\r\n");
        exp.push_str("SAMODE test6 :+r\r\n");
        exp.push_str("PRIVMSG test6 :Password accepted - you are now recognized.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn reclaim_failed_password_incorrect() {
        let u = User::new("test6", "test", None).unwrap();
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :RECLAIM test6 tset");
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn reclaim_failed_nickname_unregistered() {
        let data = test_helper(":test!test@test PRIVMSG test :RECLAIM unregistered test");
        let mut exp = "PRIVMSG test :That nick isn't registered, ".into_string();
        exp.push_str("and therefore cannot be reclaimed.\r\n");
        assert_eq!(data[], exp[]);
    }
}
