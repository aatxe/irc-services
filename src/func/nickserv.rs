use super::Functionality;
use data::BotResult;
use data::user::User;

pub struct Register {
    nickname: String,
    password: String,
    email: Option<String>,
}

impl Register {
    pub fn new(user: &str, args: Vec<&str>) -> BotResult<Register> {
        if args.len() != 2 && args.len() != 3 {
            return Err("Syntax: REGISTER password [email]".into_string())
        }
        Ok(Register {
            nickname: user.into_string(),
            password: args[1].into_string(),
            email: if args.len() == 3 {
                Some(args[2].into_string())
            } else {
                None
            }
        })
    }
}

impl Functionality for Register {
    fn do_func(&self) -> String {
        let u = User::new(self.nickname[], self.password[], self.email.as_ref().map(|s| s[]));
        if u.exists() {
            format!("{} is already registered!", u.nickname)
        } else if u.save().is_ok() {
            format!("{} has been registered. Don't forget your password!", u.nickname)
        } else {
            format!("Failed to register {} for an unknown reason.", u.nickname)
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::fs::unlink;
    use data::user::User;
    use func::test::test_helper;

    #[test]
    fn register_success() {
        let _ = unlink(&Path::new("data/nickserv/test4.json"));
        let data = test_helper(":test4!test@test PRIVMSG test :REGISTER test");
        assert_eq!(data[], "PRIVMSG test4 :test4 has been registered. Don't forget your password!\r\n");
    }

    #[test]
    fn register_failed_user_exists() {
        let u = User::new("test", "test", None);
        assert!(u.save().is_ok());
        let data = test_helper(":test!test@test PRIVMSG test :REGISTER test");
        assert_eq!(data[], "PRIVMSG test :test is already registered!\r\n");
    }
}
