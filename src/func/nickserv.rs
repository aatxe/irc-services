use super::Functionality;
use std::io::IoResult;
use data::BotResult;
use data::state::State;
use data::user::User;
use irc::server::utils::Wrapper;
use irc::data::kinds::IrcStream;

pub struct Register<'a, T> where T: IrcStream {
    server: &'a Wrapper<'a, T>,
    state: &'a State<T>,
    nickname: String,
    password: String,
    email: Option<String>,
}

impl<'a, T> Register<'a, T> where T: IrcStream {
    pub fn new(server: &'a Wrapper<'a, T>, user: &str, args: Vec<&str>, state: &'a State<T>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 && args.len() != 4 {
            return Err("Syntax: NS REGISTER password [email]".into_string())
        }
        Ok(box Register {
            server: server,
            state: state,
            nickname: user.into_string(),
            password: args[2].into_string(),
            email: if args.len() == 4 {
                Some(args[3].into_string())
            } else {
                None
            }
        } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for Register<'a, T> where T: IrcStream {
    fn do_func(&self) -> IoResult<()> {
        let user = try!(
            User::new(self.nickname[], self.password[], self.email.as_ref().map(|s| s[]))
        );
        let msg = if User::exists(self.nickname[]) {
            format!("Nickname {} is already registered!", user.nickname)
        } else if user.save().is_ok() {;
            try!(self.server.send_samode(self.nickname[], "+r", ""));
            self.state.identify(self.nickname[]);
            format!("Nickname {} has been registered. Don't forget your password!\r\n{}",
                    user.nickname, "You're now identified.")
        } else {
            format!("Failed to register {} due to an I/O issue.", user.nickname)
        };
        self.server.send_privmsg(self.nickname[], msg[])
    }
}

pub struct Identify<'a, T> where T: IrcStream {
    server: &'a Wrapper<'a, T>,
    state: &'a State<T>,
    nickname: String,
    password: String,
}

impl<'a, T> Identify<'a, T> where T: IrcStream {
    pub fn new(server: &'a Wrapper<'a, T>, user: &str, args: Vec<&str>, state: &'a State<T>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 {
            return Err("Syntax: NS IDENTIFY password".into_string())
        }
        Ok(box Identify {
            server: server,
            state: state,
            nickname: user.into_string(),
            password: args[2].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for Identify<'a, T> where T: IrcStream {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !User::exists(self.nickname[]) {
            "Your nick isn't registered."
        } else if let Ok(user) = User::load(self.nickname[]) {
            if try!(user.is_password(self.password[])) {
                try!(self.server.send_samode(self.nickname[], "+r", ""));
                self.state.identify(self.nickname[]);
                "Password accepted - you are now recognized."
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to identify due to an I/O issue."
        };
        self.server.send_privmsg(self.nickname[], msg)
    }
}

pub struct Ghost<'a, T> where T: IrcStream {
    server: &'a Wrapper<'a, T>,
    current_nick: String,
    nickname: String,
    password: String,
}

impl<'a, T> Ghost<'a, T> where T: IrcStream {
    pub fn new(server: &'a Wrapper<'a, T>, user: &str, args: Vec<&str>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err("Syntax: NS GHOST nickname password".into_string())
        }
        Ok(box Ghost {
            server: server,
            current_nick: user.into_string(),
            nickname: args[2].into_string(),
            password: args[3].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for Ghost<'a, T> where T: IrcStream {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !User::exists(self.nickname[]) {
            "That nick isn't registered, and therefore cannot be ghosted."
        } else if let Ok(user) = User::load(self.nickname[]) {
            if try!(user.is_password(self.password[])) {
                try!(self.server.send_kill(self.nickname[],
                     format!("Ghosted by {}", self.current_nick[])[]));
                try!(self.server.send_privmsg(self.nickname[], "User has been ghosted."));
                return Ok(());
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to ghost nick due to an I/O issue."
        };
        self.server.send_privmsg(self.current_nick[], msg)
    }
}

pub struct Reclaim<'a, T> where T: IrcStream {
    server: &'a Wrapper<'a, T>,
    state: &'a State<T>,
    current_nick: String,
    nickname: String,
    password: String,
}

impl<'a, T> Reclaim<'a, T> where T: IrcStream {
    pub fn new(server: &'a Wrapper<'a, T>, user: &str, args: Vec<&str>, state: &'a State<T>) -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err("Syntax: NS RECLAIM nickname password".into_string())
        }
        Ok(box Reclaim {
            server: server,
            state: state,
            current_nick: user.into_string(),
            nickname: args[2].into_string(),
            password: args[3].into_string(),
        } as Box<Functionality>)
    }
}

impl<'a, T> Functionality for Reclaim<'a, T> where T: IrcStream {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !User::exists(self.nickname[]) {
            "That nick isn't registered, and therefore cannot be reclaimed."
        } else if let Ok(user) = User::load(self.nickname[]) {
            if try!(user.is_password(self.password[])) {
                try!(self.server.send_kill(self.nickname[],
                     format!("Reclaimed by {}", self.current_nick)[]));
                try!(self.server.send_sanick(self.current_nick[], self.nickname[]));
                try!(self.server.send_samode(self.nickname[], "+r", ""));
                self.state.identify(self.nickname[]);
                try!(self.server.send_privmsg(self.nickname[],
                                           "Password accepted - you are now recognized."));
                return Ok(());
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to reclaim nick due to an I/O issue."
        };
        self.server.send_privmsg(self.current_nick[], msg)
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
        let (data, state) = test_helper(":test4!test@test PRIVMSG test :NS REGISTER test\r\n", |_| {});
        assert!(state.is_identified("test4"));
        let mut exp = "SAMODE test4 +r\r\n".into_string();
        exp.push_str("PRIVMSG test4 :Nickname test4 has been registered. ");
        exp.push_str("Don't forget your password!\r\n");
        exp.push_str("PRIVMSG test4 :You're now identified.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn register_failed_user_exists() {
        let u = User::new("test", "test", None).unwrap();
        assert!(u.save().is_ok());
        let (data, state) = test_helper(":test!test@test PRIVMSG test :NS REGISTER test\r\n", |_| {});
        assert!(!state.is_identified("test"));
        assert_eq!(data[], "PRIVMSG test :Nickname test is already registered!\r\n");
    }

    #[test]
    fn identify_succeeded() {
        let u = User::new("test5", "test", None).unwrap();
        assert!(u.save().is_ok());
        let (data, state) = test_helper(":test5!test@test PRIVMSG test :NS IDENTIFY test\r\n", |_| {});
        assert!(state.is_identified("test5"));
        let mut exp = "SAMODE test5 +r\r\n".into_string();
        exp.push_str("PRIVMSG test5 :Password accepted - you are now recognized.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn identify_failed_password_incorrect() {
        let u = User::new("test9", "test", None).unwrap();
        assert!(u.save().is_ok());
        let (data, state) = test_helper(":test9!test@test PRIVMSG test :NS IDENTIFY tset\r\n", |_| {});
        assert!(!state.is_identified("test9"));
        assert_eq!(data[], "PRIVMSG test9 :Password incorrect.\r\n");
    }

    #[test]
    fn identify_failed_nickname_unregistered() {
        let (data, state) = test_helper(":unregistered!test@test PRIVMSG test :NS IDENTIFY test\r\n", |_| {});
        assert!(!state.is_identified("unregistered"));
        assert_eq!(data[], "PRIVMSG unregistered :Your nick isn't registered.\r\n");
    }

    #[test]
    fn ghost_succeeded() {
        let u = User::new("test6", "test", None).unwrap();
        assert!(u.save().is_ok());
        let (data, _) = test_helper(":test!test@test PRIVMSG test :NS GHOST test6 test\r\n", |_| {});
        let mut exp = "KILL test6 :Ghosted by test\r\n".into_string();
        exp.push_str("PRIVMSG test6 :User has been ghosted.\r\n");
        assert_eq!(data[], exp[]);
    }


    #[test]
    fn ghost_failed_password_incorrect() {
        let u = User::new("test8", "test", None).unwrap();
        assert!(u.save().is_ok());
        let (data, _) = test_helper(":test!test@test PRIVMSG test :NS GHOST test8 tset\r\n", |_| {});
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn ghost_failed_nickname_unregistered() {
        let (data, _) = test_helper(":test!test@test PRIVMSG test :NS GHOST unregistered test\r\n", |_| {});
        let mut exp = "PRIVMSG test :That nick isn't registered, ".into_string();
        exp.push_str("and therefore cannot be ghosted.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn reclaim_succeeded() {
        let u = User::new("test11", "test", None).unwrap();
        assert!(u.save().is_ok());
        let (data, state) = test_helper(":test!test@test PRIVMSG test :NS RECLAIM test11 test\r\n", |_| {});
        assert!(state.is_identified("test11"));
        let mut exp = "KILL test11 :Reclaimed by test\r\n".into_string();
        exp.push_str("SANICK test test11\r\n");
        exp.push_str("SAMODE test11 +r\r\n");
        exp.push_str("PRIVMSG test11 :Password accepted - you are now recognized.\r\n");
        assert_eq!(data[], exp[]);
    }

    #[test]
    fn reclaim_failed_password_incorrect() {
        let u = User::new("test10", "test", None).unwrap();
        assert!(u.save().is_ok());
        let (data, state) = test_helper(":test!test@test PRIVMSG test :NS RECLAIM test10 tset\r\n", |_| {});
        assert!(!state.is_identified("test10"));
        assert_eq!(data[], "PRIVMSG test :Password incorrect.\r\n");
    }

    #[test]
    fn reclaim_failed_nickname_unregistered() {
        let (data, state) = test_helper(":test!test@test PRIVMSG test :NS RECLAIM unregistered test\r\n", |_| {});
        assert!(!state.is_identified("unregistered"));
        let mut exp = "PRIVMSG test :That nick isn't registered, ".into_string();
        exp.push_str("and therefore cannot be reclaimed.\r\n");
        assert_eq!(data[], exp[]);
    }
}
