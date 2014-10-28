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
