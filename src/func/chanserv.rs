use super::Functionality;
use std::borrow::ToOwned;
use std::old_io::IoResult;
use data::BotResult;
use data::channel::Channel;
use data::state::State;
use irc::client::server::Server;
use irc::client::server::utils::Wrapper;
use irc::client::data::kinds::{IrcReader, IrcWriter};

pub struct Register<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
}

impl<'a, T: IrcReader, U: IrcWriter> Register<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 4 {
            return Err("Syntax: CS REGISTER channel password".to_owned())
        } else if !args[2].starts_with("#") && !args[2][1..].contains("#") {
            return Err("Channels must be prefixed with a #.".to_owned())
        }
        Ok(box Register {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[2].to_owned(),
            password: args[3].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for Register<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let chan = try!(Channel::new(&self.channel, &self.password, &self.owner));
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if Channel::exists(&self.channel) {
            format!("Channel {} is already registered!", chan.name)
        } else if chan.save().is_ok() {;
            try!(self.server.send_samode(&self.channel, "+r", ""));
            try!(self.server.send_samode(&self.channel, "+qa", &self.owner));
            try!(self.server.send_join(&self.channel));
            try!(self.server.send_samode(&self.channel, "+a", self.server.config().nickname()));
            format!("Channel {} has been registered. Don't forget the password!", chan.name)
        } else {
            format!("Failed to register {} due to an I/O issue.", chan.name)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

pub struct Admin<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T: IrcReader, U: IrcWriter> Admin<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS ADMIN user channel password".to_owned())
        }
        Ok(box Admin {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[3].to_owned(),
            password: args[4].to_owned(),
            target: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for Admin<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if !self.state.is_identified(&self.target) {
            format!("{} must be identified to do that.", &self.target)
        } else if !Channel::exists(&self.channel) {
            format!("Channel {} is not registered!", &self.channel)
        } else if let Ok(mut chan) = Channel::load(&self.channel) {
            if try!(chan.is_password(&self.password)) {
                chan.admins.push(self.target.clone());
                try!(chan.save());
                try!(self.server.send_samode(&self.channel, "+a", &self.target));
                format!("{} is now an admin.", &self.target)
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to admin {} due to an I/O issue.", &self.target)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

pub struct Oper<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T: IrcReader, U: IrcWriter> Oper<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS OPER user channel password".to_owned())
        }
        Ok(box Oper {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[3].to_owned(),
            password: args[4].to_owned(),
            target: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for Oper<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if !self.state.is_identified(&self.target) {
            format!("{} must be identified to do that.", &self.target)
        } else if !Channel::exists(&self.channel) {
            format!("Channel {} is not registered!", &self.channel)
        } else if let Ok(mut chan) = Channel::load(&self.channel) {
            if try!(chan.is_password(&self.password)) {
                chan.opers.push(self.target.clone());
                try!(chan.save());
                try!(self.server.send_samode(&self.channel, "+o", &self.target));
                format!("{} is now an oper.", &self.target)
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to oper {} due to an I/O issue.", &self.target)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

pub struct Voice<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T: IrcReader, U: IrcWriter> Voice<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS VOICE user channel password".to_owned())
        }
        Ok(box Voice {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[3].to_owned(),
            password: args[4].to_owned(),
            target: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for Voice<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if !self.state.is_identified(&self.target) {
            format!("{} must be identified to do that.", &self.target)
        } else if !Channel::exists(&self.channel) {
            format!("Channel {} is not registered!", &self.channel)
        } else if let Ok(mut chan) = Channel::load(&self.channel) {
            if try!(chan.is_password(&self.password)) {
                chan.voice.push(self.target.clone());
                try!(chan.save());
                try!(self.server.send_samode(&self.channel, "+v", &self.target));
                format!("{} is now voiced.", &self.target)
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to voice {} due to an I/O issue.", &self.target)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

pub struct Mode<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
    mode: String,
}

impl<'a, T: IrcReader, U: IrcWriter> Mode<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS MODE mode channel password".to_owned())
        }
        Ok(box Mode {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[3].to_owned(),
            password: args[4].to_owned(),
            mode: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for Mode<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if !Channel::exists(&self.channel) {
            format!("Channel {} is not registered!", &self.channel)
        } else if let Ok(mut chan) = Channel::load(&self.channel) {
            if try!(chan.is_password(&self.password)) {
                chan.mode = self.mode.clone();
                try!(chan.save());
                try!(self.server.send_samode(&self.channel, &self.mode, ""));
                format!("Channel mode is now {}.", &self.mode)
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to set channel mode {} due to an I/O issue.", &self.mode)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

pub struct DeAdmin<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T: IrcReader, U: IrcWriter> DeAdmin<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS DEADMIN user channel password".to_owned())
        }
        Ok(box DeAdmin {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[3].to_owned(),
            password: args[4].to_owned(),
            target: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for DeAdmin<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if !self.state.is_identified(&self.target) {
            format!("{} must be identified to do that.", &self.target)
        } else if !Channel::exists(&self.channel) {
            format!("Channel {} is not registered!", &self.channel)
        } else if let Ok(mut chan) = Channel::load(&self.channel) {
            if try!(chan.is_password(&self.password)) {
                chan.admins.retain(|u| u != &self.target);
                try!(chan.save());
                try!(self.server.send_samode(&self.channel, "-a", &self.target));
                format!("{} is no longer an admin.", &self.target)
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to de-admin {} due to an I/O issue.", &self.target)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

pub struct DeOper<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T: IrcReader, U: IrcWriter> DeOper<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS DEOPER user channel password".to_owned())
        }
        Ok(box DeOper {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[3].to_owned(),
            password: args[4].to_owned(),
            target: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for DeOper<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if !self.state.is_identified(&self.target) {
            format!("{} must be identified to do that.", &self.target)
        } else if !Channel::exists(&self.channel) {
            format!("Channel {} is not registered!", &self.channel)
        } else if let Ok(mut chan) = Channel::load(&self.channel) {
            if try!(chan.is_password(&self.password)) {
                chan.opers.retain(|u| u != &self.target);
                try!(chan.save());
                try!(self.server.send_samode(&self.channel, "-o", &self.target));
                format!("{} is no longer an oper.", &self.target)
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to de-oper {} due to an I/O issue.", &self.target)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

pub struct DeVoice<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T: IrcReader, U: IrcWriter> DeVoice<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS DEVOICE user channel password".to_owned())
        }
        Ok(box DeVoice {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[3].to_owned(),
            password: args[4].to_owned(),
            target: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for DeVoice<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if !self.state.is_identified(&self.target) {
            format!("{} must be identified to do that.", &self.target)
        } else if !Channel::exists(&self.channel) {
            format!("Channel {} is not registered!", &self.channel)
        } else if let Ok(mut chan) = Channel::load(&self.channel) {
            if try!(chan.is_password(&self.password)) {
                chan.voice.retain(|u| u != &self.target);
                try!(chan.save());
                try!(self.server.send_samode(&self.channel, "-v", &self.target));
                format!("{} is no longer voiced.", &self.target)
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to de-voice {} due to an I/O issue.", &self.target)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

pub struct ChangeOwner<'a, T: IrcReader, U: IrcWriter> {
    server: &'a Wrapper<'a, T, U>,
    state: &'a State,
    owner: String,
    channel: String,
    password: String,
    target: String,
}

impl<'a, T: IrcReader, U: IrcWriter> ChangeOwner<'a, T, U> {
    pub fn new(server: &'a Wrapper<'a, T, U>, user: &str, args: Vec<&str>, state: &'a State)
        -> BotResult<Box<Functionality + 'a>> {
        if args.len() != 5 {
            return Err("Syntax: CS CHOWN user channel password".to_owned())
        }
        Ok(box ChangeOwner {
            server: server,
            state: state,
            owner: user.to_owned(),
            channel: args[3].to_owned(),
            password: args[4].to_owned(),
            target: args[2].to_owned(),
        } as Box<Functionality>)
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Functionality for ChangeOwner<'a, T, U> {
    fn do_func(&self) -> IoResult<()> {
        let msg = if !self.state.is_identified(&self.owner) {
            format!("You must be identify as {} to do that.", &self.owner)
        } else if !self.state.is_identified(&self.target) && &self.target[..] != "Pidgey" {
            format!("{} must be identified to do that.", &self.target)
        } else if !Channel::exists(&self.channel) {
            format!("Channel {} is not registered!", &self.channel)
        } else if let Ok(mut chan) = Channel::load(&self.channel) {
            if try!(chan.is_password(&self.password)) {
                chan.owner = self.target.clone();
                try!(chan.save());
                try!(self.server.send_samode(&self.channel, "+q", &self.target));
                format!("{} is now the channel owner.", &self.target)
            } else {
                format!("Password incorrect.")
            }
        } else {
            format!("Failed to change owner to {} due to an I/O issue.", &self.target)
        };
        self.server.send_notice(&self.owner, &msg)
    }
}

#[cfg(test)]
mod test {
    use std::borrow::ToOwned;
    use std::old_io::fs::unlink;
    use data::channel::Channel;
    use func::test::test_helper;

    #[test]
    fn register_succeeded() {
        let _ = unlink(&Path::new("data/chanserv/#test4.json"));
        let (data, _) = test_helper(
            ":test2!test@test PRIVMSG test :CS REGISTER #test4 test\r\n", |state| {
            state.identify("test2");
        });
        let exp = "SAMODE #test4 +r\r\n\
                   SAMODE #test4 +qa test2\r\n\
                   JOIN #test4\r\n\
                   SAMODE #test4 +a test\r\n\
                   NOTICE test2 :Channel #test4 has been registered. \
                   Don't forget the password!\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn register_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS REGISTER #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn register_failed_channel_exists() {
        let ch = Channel::new("#test", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS REGISTER #test test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #test is already registered!\r\n");
    }

    #[test]
    fn admin_succeeded() {
        let ch = Channel::new("#test5", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS ADMIN test2 #test5 test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(Channel::load("#test5").unwrap().admins, vec!("test2".to_owned()));
        let exp = "SAMODE #test5 +a test2\r\n\
                   NOTICE test :test2 is now an admin.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn admin_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS ADMIN test2 #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn admin_failed_target_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS ADMIN test2 #test test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :test2 must be identified to do that.\r\n");
    }

    #[test]
    fn admin_failed_channel_unregistered() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS ADMIN test2 #unregistered test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn admin_failed_password_incorrect() {
        let ch = Channel::new("#test12", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS ADMIN test2 #test12 wrong\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n");
    }

    #[test]
    fn oper_succeeded() {
        let ch = Channel::new("#test6", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS OPER test2 #test6 test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(Channel::load("#test6").unwrap().opers, vec!("test2".to_owned()));
        let exp = "SAMODE #test6 +o test2\r\n\
                   NOTICE test :test2 is now an oper.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn oper_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS OPER test2 #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn oper_failed_target_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS OPER test2 #test test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :test2 must be identified to do that.\r\n");
    }

    #[test]
    fn oper_failed_channel_unregistered() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS OPER test2 #unregistered test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn oper_failed_password_incorrect() {
        let ch = Channel::new("#test13", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS OPER test2 #test13 wrong\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n");
    }

    #[test]
    fn voice_succeeded() {
        let ch = Channel::new("#test7", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS VOICE test2 #test7 test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(Channel::load("#test7").unwrap().voice, vec!("test2".to_owned()));
        let exp = "SAMODE #test7 +v test2\r\n\
                   NOTICE test :test2 is now voiced.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn voice_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS VOICE test2 #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn voice_failed_target_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS VOICE test2 #test test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :test2 must be identified to do that.\r\n");
    }

    #[test]
    fn voice_failed_channel_unregistered() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS VOICE test2 #unregistered test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn voice_failed_password_incorrect() {
        let ch = Channel::new("#test14", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS VOICE test2 #test14 wrong\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n");
    }

    #[test]
    fn mode_succeeded() {
        let ch = Channel::new("#test15", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS MODE +i #test15 test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        let exp = "SAMODE #test15 +i\r\n\
                   NOTICE test :Channel mode is now +i.\r\n";
        assert_eq!(&data[..], exp)
    }

    #[test]
    fn mode_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS MODE +i #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn mode_failed_channel_unregistered() {
        let (data, _) = test_helper(":test!test@test PRIVMSG test :CS MODE +i #unregistered test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn mode_failed_password_incorrect() {
        let ch = Channel::new("#test16", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS MODE +i #test16 wrong\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n");
    }

    #[test]
    fn deadmin_succeeded() {
        let mut ch = Channel::new("#test17", "test", "test").unwrap();
        ch.admins.push("test2".to_owned());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEADMIN test2 #test17 test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert!(Channel::load("#test17").unwrap().admins.is_empty());
        let exp = "SAMODE #test17 -a test2\r\n\
                   NOTICE test :test2 is no longer an admin.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn deadmin_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEADMIN test2 #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn deadmin_failed_target_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEADMIN test2 #test test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :test2 must be identified to do that.\r\n");
    }

    #[test]
    fn deadmin_failed_channel_unregistered() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEADMIN test2 #unregistered test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn deadmin_failed_incorrect_password() {
        let mut ch = Channel::new("#test18", "test", "test").unwrap();
        ch.admins.push("test2".to_owned());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEADMIN test2 #test18 wrong\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n")
    }

    #[test]
    fn deoper_succeeded() {
        let mut ch = Channel::new("#test19", "test", "test").unwrap();
        ch.opers.push("test2".to_owned());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEOPER test2 #test19 test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert!(Channel::load("#test19").unwrap().opers.is_empty());
        let exp = "SAMODE #test19 -o test2\r\n\
                   NOTICE test :test2 is no longer an oper.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn deoper_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEOPER test2 #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn deoper_failed_target_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEOPER test2 #test test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :test2 must be identified to do that.\r\n");
    }

    #[test]
    fn deoper_failed_channel_unregistered() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEOPER test2 #unregistered test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn deoper_failed_incorrect_password() {
        let mut ch = Channel::new("#test20", "test", "test").unwrap();
        ch.opers.push("test2".to_owned());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEOPER test2 #test20 wrong\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n");
    }

    #[test]
    fn devoice_succeeded() {
        let mut ch = Channel::new("#test21", "test", "test").unwrap();
        ch.voice.push("test2".to_owned());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEVOICE test2 #test21 test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert!(Channel::load("#test21").unwrap().voice.is_empty());
        let exp = "SAMODE #test21 -v test2\r\n\
                   NOTICE test :test2 is no longer voiced.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn devoice_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEVOICE test2 #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn devoice_failed_target_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEVOICE test2 #test test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :test2 must be identified to do that.\r\n");
    }

    #[test]
    fn devoice_failed_channel_unregistered() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEVOICE test2 #unregistered test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn devoice_failed_incorrect_password() {
        let mut ch = Channel::new("#test22", "test", "test").unwrap();
        ch.voice.push("test2".to_owned());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS DEVOICE test2 #test22 wrong\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n")
    }

    #[test]
    fn chown_succeeded() {
        let ch = Channel::new("#test24", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS CHOWN test2 #test24 test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&Channel::load("#test24").unwrap().owner[..], "test2");
        let exp = "SAMODE #test24 +q test2\r\n\
                   NOTICE test :test2 is now the channel owner.\r\n";
        assert_eq!(&data[..], exp);
    }

    #[test]
    fn chown_failed_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS CHOWN test2 #test test\r\n", |_| {}
        );
        assert_eq!(&data[..], "NOTICE test :You must be identify as test to do that.\r\n");
    }

    #[test]
    fn chown_failed_target_not_identified() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS CHOWN test2 #test test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(&data[..], "NOTICE test :test2 must be identified to do that.\r\n");
    }

    #[test]
    fn chown_failed_channel_unregistered() {
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS CHOWN test2 #unregistered test\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Channel #unregistered is not registered!\r\n");
    }

    #[test]
    fn chown_failed_password_incorrect() {
        let ch = Channel::new("#test25", "test", "test").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(
            ":test!test@test PRIVMSG test :CS CHOWN test2 #test25 wrong\r\n", |state| {
            state.identify("test");
            state.identify("test2");
        });
        assert_eq!(&data[..], "NOTICE test :Password incorrect.\r\n");
    }
}
