use super::Functionality;
use data::BotResult;
use data::user::User;

pub struct Register {
    nickname: String,
    password: String,
    email: Option<String>,
}

impl Register {
    pub fn new<'a>(user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 2 && args.len() != 3 {
            return Err("Syntax: REGISTER password [email]".into_string())
        }
        Ok(box Register {
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

impl Functionality for Register {
    fn do_func(&self) -> String {
        let user = User::new(self.nickname[], self.password[], self.email.as_ref().map(|s| s[]));
        if User::exists(self.nickname[]) {
            format!("Nickname {} is already registered!", user.nickname)
        } else if user.save().is_ok() {
            format!("Nickname {} has been registered. Don't forget your password!", user.nickname)
        } else {
            format!("Failed to register {} for an unknown reason.", user.nickname)
        }
    }
}

pub struct Identify {
    nickname: String,
    password: String,
}

impl Identify {
    pub fn new<'a>(user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 2 {
            return Err("Syntax: IDENTIFY password".into_string())
        }
        Ok(box Identify {
            nickname: user.into_string(),
            password: args[1].into_string(),
        } as Box<Functionality>)
    }
}

impl Functionality for Identify {
    fn do_func(&self) -> String {
        if !User::exists(self.nickname[]) {
            format!("Your nick isn't registered.")
        } else if let Ok(user) = User::load(self.nickname[]) {
            if user.is_password(self.password[]) {
                format!("Password accepted - you are now recognized.")
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to identify for an unknown reason.")
        }
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
        let u = User::new("test", "test", None);
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :IDENTIFY test");
        assert_eq!(data[], "PRIVMSG test :Password accepted - you are now recognized.\r\n");
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
