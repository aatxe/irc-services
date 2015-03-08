use super::Functionality;
use std::borrow::ToOwned;
use std::io::Result;
use data::BotResult;
use data::state::State;
use data::user::User;
use irc::client::prelude::*;

pub struct Register<'a, T: IrcRead, U: IrcWrite> {
    server: &'a ServerExt<'a, T, U>,
    state: &'a State,
    nickname: String,
    password: String,
    email: Option<String>,
}

impl<'a, T: IrcRead, U: IrcWrite> Register<'a, T, U> {
    pub fn new(server: &'a ServerExt<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 && args.len() != 4 {
            return Err("Syntax: NS REGISTER password [email]".to_owned())
        }
        Ok(box Register {
            server: server,
            state: state,
            nickname: user.to_owned(),
            password: args[2].to_owned(),
            email: if args.len() == 4 {
                Some(args[3].to_owned())
            } else {
                None
            }
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Functionality for Register<'a, T, U> {
    fn do_func(&self) -> Result<()> {
        let user = try!(
            User::new(&self.nickname, &self.password, self.email.as_ref().map(|s| &s[..]))
        );
        let msg = if User::exists(&self.nickname) {
            format!("Nickname {} is already registered!", user.nickname)
        } else if user.save().is_ok() {;
            try!(self.server.send_samode(&self.nickname, "+r", ""));
            self.state.identify(&self.nickname);
            format!("Nickname {} has been registered. Don't forget your password!\r\n{}",
                    user.nickname, "You're now identified.")
        } else {
            format!("Failed to register {} due to an I/O issue.", user.nickname)
        };
        self.server.send_notice(&self.nickname, &msg)
    }
}

pub struct Identify<'a, T: IrcRead, U: IrcWrite> {
    server: &'a ServerExt<'a, T, U>,
    state: &'a State,
    nickname: String,
    password: String,
}

impl<'a, T: IrcRead, U: IrcWrite> Identify<'a, T, U> {
    pub fn new(server: &'a ServerExt<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 3 {
            return Err("Syntax: NS IDENTIFY password".to_owned())
        }
        Ok(box Identify {
            server: server,
            state: state,
            nickname: user.to_owned(),
            password: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Functionality for Identify<'a, T, U> {
    fn do_func(&self) -> Result<()> {
        let msg = if !User::exists(&self.nickname) {
            "Your nick isn't registered."
        } else if let Ok(user) = User::load(&self.nickname) {
            if try!(user.is_password(&self.password)) {
                try!(self.server.send_samode(&self.nickname, "+r", ""));
                self.state.identify(&self.nickname);
                "Password accepted - you are now recognized."
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to identify due to an I/O issue."
        };
        self.server.send_notice(&self.nickname, msg)
    }
}

pub struct Ghost<'a, T: IrcRead, U: IrcWrite> {
    server: &'a ServerExt<'a, T, U>,
    current_nick: String,
    nickname: String,
    password: String,
}

impl<'a, T: IrcRead, U: IrcWrite> Ghost<'a, T, U> {
    pub fn new(server: &'a ServerExt<'a, T, U>, user: &str, args: Vec<&str>)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err("Syntax: NS GHOST nickname password".to_owned())
        }
        Ok(box Ghost {
            server: server,
            current_nick: user.to_owned(),
            nickname: args[2].to_owned(),
            password: args[3].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Functionality for Ghost<'a, T, U> {
    fn do_func(&self) -> Result<()> {
        let msg = if !User::exists(&self.nickname) {
            "That nick isn't registered, and therefore cannot be ghosted."
        } else if let Ok(user) = User::load(&self.nickname) {
            if try!(user.is_password(&self.password)) {
                try!(self.server.send_kill(&self.nickname,
                     &format!("Ghosted by {}", &self.current_nick)));
                try!(self.server.send_notice(&self.nickname, "User has been ghosted."));
                return Ok(());
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to ghost nick due to an I/O issue."
        };
        self.server.send_notice(&self.current_nick, msg)
    }
}

pub struct Reclaim<'a, T: IrcRead, U: IrcWrite> {
    server: &'a ServerExt<'a, T, U>,
    state: &'a State,
    current_nick: String,
    nickname: String,
    password: String,
}

impl<'a, T: IrcRead, U: IrcWrite> Reclaim<'a, T, U> {
    pub fn new(server: &'a ServerExt<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err("Syntax: NS RECLAIM nickname password".to_owned())
        }
        Ok(box Reclaim {
            server: server,
            state: state,
            current_nick: user.to_owned(),
            nickname: args[2].to_owned(),
            password: args[3].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Functionality for Reclaim<'a, T, U> {
    fn do_func(&self) -> Result<()> {
        let msg = if !User::exists(&self.nickname) {
            "That nick isn't registered, and therefore cannot be reclaimed."
        } else if let Ok(user) = User::load(&self.nickname) {
            if try!(user.is_password(&self.password)) {
                try!(self.server.send_kill(&self.nickname,
                     &format!("Reclaimed by {}", self.current_nick)));
                try!(self.server.send_sanick(&self.current_nick, &self.nickname));
                try!(self.server.send_samode(&self.nickname, "+r", ""));
                self.state.identify(&self.nickname);
                try!(self.server.send_notice(&self.nickname,
                                           "Password accepted - you are now recognized."));
                return Ok(());
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to reclaim nick due to an I/O issue."
        };
        self.server.send_notice(&self.current_nick, msg)
    }
}

pub struct ChangePassword<'a, T: IrcRead, U: IrcWrite> {
    server: &'a ServerExt<'a, T, U>,
    user: String,
    password: String,
    new_password: String,
}

impl<'a, T: IrcRead, U: IrcWrite> ChangePassword<'a, T, U> {
    pub fn new(server: &'a ServerExt<'a, T, U>, user: &str, args: Vec<&str>)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err("Syntax: NS CHPASS old_password new_password".to_owned())
        }
        Ok(box ChangePassword {
            server: server,
            user: user.to_owned(),
            password: args[2].to_owned(),
            new_password: args[3].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Functionality for ChangePassword<'a, T, U> {
    fn do_func(&self) -> Result<()> {
        let msg = if !User::exists(&self.user) {
            "This nick isn't registered, and therefore doesn't have a password to change."
        } else if let Ok(mut user) = User::load(&self.user) {
            if try!(user.is_password(&self.password)) {
                try!(user.update_password(&self.new_password));
                try!(user.save());
                "Your password has been changed. Don't forget it!"
            } else {
                "Password incorrect."
            }
        } else {
            "Failed to change password due to an I/O issue."
        };
        self.server.send_notice(&self.user, msg)
    }
}

#[cfg(test)]
mod test {
    use std::fs::remove_file;
    use std::path::Path;
    use data::user::User;
    use func::test::test_helper;

    #[test]
    fn register_succeeded() {
        let _ = remove_file(Path::new("data/nickserv/test4.json"));
        let (data, state) = test_helper(
            ":test4!test@test PRIVMSG test :NS REGISTER test\r\n", |_| {}
        );
        assert!(state.is_identified("test4"));
        let exp = "SAMODE test4 +r\r\n\
                   NOTICE test4 :Nickname test4 has been registered. \
                   Don't forget your password!\r\n\
                   NOTICE test4 :You're now identified.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn register_failed_user_exists() {
        let u = User::new("test", "test", None).unwrap();
        u.save().unwrap();
        let (data, state) = test_helper(
            ":test!test@test PRIVMSG test :NS REGISTER test\r\n", |_| {}
        );
        assert!(!state.is_identified("test"));
        assert_eq!(&data[..], "NOTICE test :Nickname test is already registered!\r\n");
    }

    #[test]
    fn identify_succeeded() {
        let u = User::new("test5", "test", None).unwrap();
        u.save().unwrap();
        let (data, state) = test_helper(
            ":test5!test@test PRIVMSG test :NS IDENTIFY test\r\n", |_| {}
        );
        assert!(state.is_identified("test5"));
        let exp = "SAMODE test5 +r\r\n\
                   NOTICE test5 :Password accepted - you are now recognized.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn identify_failed_password_incorrect() {
        let u = User::new("test9", "test", None).unwrap();
        u.save().unwrap();
        let (data, state) = test_helper(
            ":test9!test@test PRIVMSG test :NS IDENTIFY tset\r\n", |_| {}
        );
        assert!(!state.is_identified("test9"));
        assert_eq!(&data[..], "NOTICE test9 :Password incorrect.\r\n");
    }

    #[test]
    fn identify_failed_nickname_unregistered() {
        let (data, state) = test_helper(
            ":unregistered!test@test PRIVMSG test :NS IDENTIFY test\r\n", |_| {}
        );
        assert!(!state.is_identified("unregistered"));
        assert_eq!(&data[..], "NOTICE unregistered :Your nick isn't registered.\r\n");
    }

    #[test]
    fn ghost_succeeded() {
        let u = User::new("test6", "test", None).unwrap();
        u.save().unwrap();
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :NS GHOST test6 test\r\n", |_| {}
        );
        let exp = "KILL test6 :Ghosted by test\r\nNOTICE test6 :User has been ghosted.\r\n";
        assert_eq!(&data[..], exp);
    }


    #[test]
    fn ghost_failed_password_incorrect() {
        let u = User::new("test8", "test", None).unwrap();
        u.save().unwrap();
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :NS GHOST test8 tset\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n");
    }

    #[test]
    fn ghost_failed_nickname_unregistered() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :NS GHOST unregistered test\r\n", |_| {}
        );
        let exp = "NOTICE test :That nick isn't registered, and therefore cannot be ghosted.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn reclaim_succeeded() {
        let u = User::new("test11", "test", None).unwrap();
        u.save().unwrap();
        let (data, state) = test_helper(
            ":test!test@test PRIVMSG test :NS RECLAIM test11 test\r\n", |_| {}
        );
        assert!(state.is_identified("test11"));
        let exp = "KILL test11 :Reclaimed by test\r\n\
                   SANICK test test11\r\n\
                   SAMODE test11 +r\r\n\
                   NOTICE test11 :Password accepted - you are now recognized.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn reclaim_failed_password_incorrect() {
        let u = User::new("test10", "test", None).unwrap();
        u.save().unwrap();
        let (data, state) = test_helper(
            ":test!test@test PRIVMSG test :NS RECLAIM test10 tset\r\n", |_| {}
        );
        assert!(!state.is_identified("test10"));
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n");
    }

    #[test]
    fn reclaim_failed_nickname_unregistered() {
        let (data, state) = test_helper(
            ":test!test@test PRIVMSG test :NS RECLAIM unregistered test\r\n", |_| {}
        );
        assert!(!state.is_identified("unregistered"));
        let exp = "NOTICE test :That nick isn't registered, and therefore cannot be reclaimed.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn chpass_succeeded() {
        let u = User::new("test13", "test", None).unwrap();
        u.save().unwrap();
        let (data, _) = test_helper(
            ":test13!test@test PRIVMSG test :NS CHPASS test blahblah\r\n", |_| {}
        );
        let u = User::load("test13").unwrap();
        assert!(u.is_password("blahblah").unwrap());
        let exp = "NOTICE test13 :Your password has been changed. Don't forget it!\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn chpass_failed_password_incorrect() {
        let u = User::new("test12", "test", None).unwrap();
        u.save().unwrap();
        let (data, _) = test_helper(
            ":test12!test@test PRIVMSG test :NS CHPASS tset blahblah\r\n", |_| {}
        );
        let exp = "NOTICE test12 :Password incorrect.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn chpass_failed_nickname_unregistered() {
        let (data, _) = test_helper(
            ":unregistered!test@test PRIVMSG test :NS CHPASS blah blahblah\r\n", |_| {}
        );
        let exp = "NOTICE unregistered :This nick isn't registered, and therefore doesn't have a \
                   password to change.\r\n";
        assert_eq!(&data[..], exp);
    }
}
