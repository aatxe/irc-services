extern crate irc;

use std::io::IoResult;
use std::io::fs::walk_dir;
use data::channel::Channel;
#[cfg(feature = "democracy")] use data::democracy::Democracy;
#[cfg(feature = "democracy")] use data::democracy::VoteResult::{VotePassed, VoteFailed};
#[cfg(feature = "democracy")] use data::democracy::VotingResult::{VoteIssued, InvalidVote};
#[cfg(feature = "democracy")] use data::democracy::VotingResult::NoSuchProposal;
#[cfg(feature = "derp")] use data::derp::DerpCounter;
#[cfg(feature = "resistance")] use data::resistance::Resistance;
use data::state::State;
use irc::server::Server;
use irc::server::utils::Wrapper;
use irc::data::kinds::IrcStream;

mod chanserv;
mod nickserv;

pub fn process<'a, T>(server: &'a Wrapper<'a, T>, source: &str, command: &str, args: &[&str],
                      state: &'a State) -> IoResult<()> where T: IrcStream {
    let user = source.find('!').map_or("", |i| source[..i]);
    if let ("PRIVMSG", [chan, msg]) = (command, args) {
        if msg.starts_with("!") {
            if try!(do_resistance(server, user, msg, chan, state)) {
                return Ok(());
            } else if msg.starts_with("!derp") &&
                      try!(do_derp(server, if chan.starts_with("#") { chan } else { user }, msg)) {
                return Ok(());
            }
        }
        if chan.starts_with("#") { return do_democracy(server, user, msg, chan, state); }
        let tokens: Vec<_> = msg.split_str(" ").collect();
        let res = if args.len() > 1 && upper_case(tokens[0])[] == "NS" {
            let cmd: String = upper_case(tokens[1]);
            match cmd[] {
                "REGISTER" => nickserv::Register::new(server, user, tokens, state),
                "IDENTIFY" => nickserv::Identify::new(server, user, tokens, state),
                "GHOST"    => nickserv::Ghost::new(server, user,tokens),
                "RECLAIM"  => nickserv::Reclaim::new(server, user, tokens, state),
                "CHPASS"   => nickserv::ChangePassword::new(server, user, tokens),
                _          => Err(format!("{} is not a valid command.", tokens[1])),
            }
        } else if args.len() > 1 && upper_case(tokens[0])[] == "CS" {
            let cmd: String = upper_case(tokens[1]);
            match cmd[] {
                "REGISTER" => chanserv::Register::new(server, user, tokens, state),
                "ADMIN"    => chanserv::Admin::new(server, user, tokens, state),
                "OPER"     => chanserv::Oper::new(server, user, tokens, state),
                "VOICE"    => chanserv::Voice::new(server, user, tokens, state),
                "MODE"     => chanserv::Mode::new(server, user, tokens, state),
                "DEADMIN"  => chanserv::DeAdmin::new(server, user, tokens, state),
                "DEOPER"   => chanserv::DeOper::new(server, user, tokens, state),
                "DEVOICE"  => chanserv::DeVoice::new(server, user, tokens, state),
                "CHOWN"    => chanserv::ChangeOwner::new(server, user, tokens, state),
                _          => Err(format!("{} is not a valid command.", tokens[1])),
            }
        } else {
            Err("Commands must be prefixed by CS or NS.".into_string())
        };
        if let Err(msg) = res {
            try!(server.send_notice(user, msg[]));
        } else {
            try!(res.unwrap().do_func())
        }
    } else if let ("NOTICE", [_, suffix]) = (command, args) {
        if suffix.starts_with("***") {
            try!(server.identify());
        }
    } else if let ("376", _) = (command, args) {
        try!(start_up(server, state));
    } else if let ("422", _) = (command, args) {
        try!(start_up(server, state));
    } else if let ("TOPIC", [chan, message]) = (command, args) {
        if let Ok(mut channel) = Channel::load(chan) {
            channel.topic = message.into_string();
            try!(channel.save());
        }
    } else if let ("JOIN", [chan]) = (command, args){
        if let Ok(channel) = Channel::load(chan) {
            let mode = if channel.owner[] == user {
                "+qa"
            } else if channel.admins[].contains(&user.into_string()) {
                "+a"
            } else if channel.opers[].contains(&user.into_string()) {
                "+o"
            } else if channel.voice[].contains(&user.into_string()) {
                "+v"
            } else {
                ""
            };
            if state.is_identified(user) && mode.len() > 0 {
                try!(server.send_samode(chan, mode[], user[]));
            }
        }
    } else if let ("QUIT", _) = (command, args) {
        state.remove(user);
    } else if let ("MODE", [chan, "+v", user]) = (command, args) {
        try!(democracy_process_hook(server, "+v", user, chan, state));
    } else if let ("MODE", [chan, "-v", user]) = (command, args) {
        try!(democracy_process_hook(server, "-v", user, chan, state));
    }
    Ok(())
}

pub trait Functionality {
    fn do_func(&self) -> IoResult<()>;
}

fn start_up<T>(server: &Wrapper<T>, state: &State) -> IoResult<()> where T: IrcStream {
    try!(server.send_oper(server.config().nickname[],
                          server.config().options[format!("oper-pass")].clone()[]));
    let mut chans: Vec<String> = Vec::new();
    for path in try!(walk_dir(&Path::new("data/chanserv/"))) {
        let path_str = path.as_str().unwrap();
        let chan = path_str.find('.').map_or(String::new(), |i| path_str[14..i].into_string());
        if chan[] != "" {
            chans.push(chan);
        }
    }
    let mut join_line = String::new();
    for chan in chans.iter() {
        if join_line.len() < 40 && join_line.len() > 0 {
            join_line.push_str(",");
            join_line.push_str(chan[]);
        } else if join_line.len() == 0 {
            join_line.push_str(chan[]);
        } else {
            try!(server.send_join(join_line[]));
            join_line = chan.clone();
        }
    }
    try!(server.send_join(join_line[]));
    for chan in chans.iter() {
        new_voting_booth(chan[], state);
        try!(server.send_samode(chan[], "+a", server.config().nickname[]));
        let ch = try!(Channel::load(chan[]));
        if ch.topic.len() != 0 {
            try!(server.send_topic(chan[], ch.topic[]));
        }
        if ch.mode.len() != 0 {
            try!(server.send_samode(chan[], ch.mode[], ""));
        }
    }
    Ok(())
}

#[cfg(feature = "resistance")]
pub fn do_resistance<'a, T>(server: &'a Wrapper<'a, T>, user: &str, message: &str, chan: &str,
                            state: &State) -> IoResult<bool> where T: IrcStream {
    let mut games = state.get_games();
    let mut remove_game = false;
    if let Some(game) = games.get_mut(&chan.into_string()) {
        if message.starts_with("!join") {
            try!(game.add_player(server, user));
            return Ok(true)
        } else if message.starts_with("!start") {
            try!(game.start(server));
            return Ok(true)
        } else if message.starts_with("!propose ") {
            try!(game.propose_mission(server, user, message[9..]));
            return Ok(true)
        } else if message.starts_with("!vote ") {
            try!(game.cast_proposal_vote(server, user, message[6..]));
            if !game.is_complete() {
                return Ok(true)
            }
            remove_game = true;
        } else if message.starts_with("!drop") && game.is_leader(user) {
            try!(server.send_privmsg(chan, "The game of Resistance has been dropped."));
            remove_game = true;
        }
        if !remove_game { return Ok(false) }
    }
    if remove_game { games.remove(&chan.into_string()); return Ok(true) }
    if message.starts_with("!resistance") && chan.starts_with("#") {
        let game = Resistance::new_game(user, chan);
        games.insert(chan.into_string(), game);
        try!(server.send_privmsg(chan, "Players may now join the game. Use `!start` to start."));
        return Ok(true)
    } else if message.starts_with("!resistance") {
        try!(server.send_privmsg(user, "You cannot start a game in a private message."));
        return Ok(true)
    } else if !chan.starts_with("#") && message.starts_with("!vote ") {
        let tokens: Vec<_> = message[6..].split_str(" ").collect();
        if tokens.len() != 2 {
            try!(server.send_privmsg(user, "You must vote like so: `!vote #chan <yea/nay>`."))
            return Ok(true)
        }
        if let Some(game) = games.get_mut(&tokens[0].into_string()) {
            try!(game.cast_mission_vote(server, user, tokens[1]));
            if !game.is_complete() {
                return Ok(true)
            }
            remove_game = true;
        } else {
            try!(server.send_privmsg(user, "There's no game on that channel."))
            return Ok(true)
        }
        if remove_game { games.remove(&tokens[0].into_string()); return Ok(true) }
    }
    Ok(false)
}

#[cfg(not(feature = "resistance"))]
pub fn do_resistance<'a, T>(_: &Wrapper<'a, T>, _: &str, _: &str, _: &str, _: &State)
    -> IoResult<bool> where T: IrcStream {
    Ok(false)
}

#[cfg(feature = "derp")]
pub fn do_derp<'a, T>(server: &Wrapper<'a, T>, resp: &str, msg: &str) -> IoResult<bool>
    where T: IrcStream {
    let dc = DerpCounter::load();
    if let Ok(mut counter) = dc {
        if msg.starts_with("!derp++") { counter.increment(); }
        if let Ok(()) = counter.save() {
            let (verb, plural) = if counter.derps() == 1 { ("has", "") } else { ("have", "s") };
            try!(server.send_privmsg(resp,
                 format!("There {} been {} derp{}.", verb, counter.derps(), plural)[])
            );
            return Ok(true);
        }
    }
    try!(server.send_privmsg(resp, "Something went wrong with the Derp Counter."));
    Ok(true)
}

#[cfg(not(feature = "derp"))]
pub fn do_derp<'a, T>(_: &Wrapper<'a, T>, _: &str, _: &str) -> IoResult<bool> where T: IrcStream {
    Ok(false)
}

#[cfg(feature = "democracy")]
pub fn new_voting_booth(chan: &str, state: &State) {
   state.get_votes().insert(chan.into_string(), Democracy::new());
}

#[cfg(not(feature = "democracy"))]
pub fn new_voting_booth(_: &str, _: &State) {}

#[cfg(feature = "democracy")]
pub fn democracy_process_hook<'a, T>(server: &'a Wrapper<'a, T>, msg: &str, user: &str, chan: &str, 
                                     state: &State) -> IoResult<()> where T: IrcStream {
    if Channel::exists(chan) {
        if msg == "+v" && state.is_identified(user) {
            if let Ok(mut channel) = Channel::load(chan) {
                channel.voice.push(user.into_string());
                try!(channel.save());
            }
        } else if msg == "-v" {
            if let Ok(mut channel) = Channel::load(chan) {
                channel.voice.retain(|u| u[] != user);
                try!(channel.save());
            }
        } else {
            try!(server.send_samode(chan, "-v", user));
        }
    }
    Ok(())
}

#[cfg(not(feature = "democracy"))]
pub fn democracy_process_hook<'a, T>(_: &'a Wrapper<'a, T>, _: &str, _: &str, _: &str, _: &State)
    -> IoResult<()> where T: IrcStream {
    Ok(())
}

#[cfg(feature = "democracy")]
pub fn do_democracy<'a, T>(server: &'a Wrapper<'a, T>, user: &str, message: &str, chan: &str,
                           state: &State) -> IoResult<()> where T: IrcStream {
    if message.starts_with(".propose") || message.starts_with(".vote") {
        if !state.is_identified(user) {
            return server.send_privmsg(chan, "You must be identified to do that.");
        } else if !state.is_voiced(user, chan) {
            return server.send_privmsg(chan, "You must be voiced to do that.");
        }
    }
    let mut votes = state.get_votes();
    if let Some(democracy) = votes.get_mut(&chan.into_string()) {
        if message.starts_with(".propose topic ") {
            let id = democracy.propose("topic", message[15..]);
            if let Some(id) = id {
                try!(server.send_privmsg(chan, format!("Proposal {} is live.", id)[]));
            }
            return Ok(())
        }
        let tokens = {
            let mut tmp: Vec<&str> = message.split_str(" ").collect();
            tmp.retain(|t| t.len() != 0);
            tmp
        };
        match tokens[] {
            [".propose", proposal, parameter] => {
                let id = democracy.propose(proposal, parameter);
                if let Some(id) = id {
                    try!(server.send_privmsg(chan, format!("Proposal {} is live.", id)[]));
                } else {
                    try!(server.send_privmsg(chan, 
                         format!("{} is not a valid option for a proposal.", proposal)[]));
                }
            },
            [".vote", proposal_id, vote] => {
                let proposal_id = from_str(proposal_id).unwrap_or(0u8);
                if democracy.has_voted(proposal_id, user) {
                    return server.send_privmsg(chan, "You've already voted in that proposal.");
                }
                let res = democracy.vote(proposal_id, vote, user);
                let msg = match res {
                    VoteIssued => format!("Vote issued."),
                    InvalidVote => format!("{} is not a valid vote.", vote),
                    NoSuchProposal => format!("{} is not a valid proposal.", proposal_id),
                };
                try!(server.send_privmsg(chan, msg[]));
                let voting_pop = if democracy.is_full_vote(proposal_id) {
                    state.get_voting_pop(chan)
                } else {
                    state.get_online_voting_pop(chan)
                };
                try!(match democracy.get_result_of_vote(proposal_id, voting_pop) {
                    VotePassed(proposal) => proposal.enact(server, chan),
                    VoteFailed => server.send_privmsg(chan, 
                                  format!("Failed to pass proposal {}.", proposal_id)[]),

                    _ => Ok(())
                })
            },
            _ => ()
        }
    }
    Ok(())
}

#[cfg(not(feature = "democracy"))]
pub fn do_democracy<'a, T>(_: &'a Wrapper<'a, T>, _: &str, _: &str, _: &str, _: &State)
    -> IoResult<()> where T: IrcStream {
    Ok(())
}

fn upper_case(string: &str) -> String {
    string.chars().map(|c| c.to_uppercase()).collect()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::io::{MemReader, MemWriter};
    #[cfg(feature = "derp")] use std::io::fs::unlink;
    use data::channel::Channel;
    use data::state::State;
    use irc::conn::{Connection, IoStream};
    use irc::data::Config;
    use irc::server::{IrcServer, Server};
    use irc::server::utils::Wrapper;

    pub fn test_helper(input: &str, state_hook: |&State| -> ()) -> (String, State) {
        let server = IrcServer::from_connection(Config {
                owners: vec!["test".into_string()],
                nickname: "test".into_string(),
                username: "test".into_string(),
                realname: "test".into_string(),
                password: String::new(),
                server: "irc.fyrechat.net".into_string(),
                port: 6667,
                use_ssl: false,
                channels: vec!["#test".into_string(), "#test2".into_string()],
                options: {
                    let mut map = HashMap::new();
                    map.insert("oper-pass".into_string(), "test".into_string());
                    map
                }
            },
            Connection::new(
                IoStream::new(MemWriter::new(), MemReader::new(input.as_bytes().to_vec()))
            )
        );
        let state = State::new();
        state_hook(&state);
        for message in server.iter() {
            println!("{}", message);
            let mut args = Vec::new();
            let msg_args: Vec<_> = message.args.iter().map(|s| s[]).collect();
            args.push_all(msg_args[]);
            if let Some(ref suffix) = message.suffix {
                args.push(suffix[])
            }
            let source = message.prefix.unwrap_or(String::new());
            super::process(&Wrapper::new(&server), source[], message.command[], args[],
                           &state).unwrap();
        }
        (String::from_utf8(server.conn().stream().value()).unwrap(), state)
    }

    #[test]
    fn commands_must_be_prefxed() {
        let (data, _) = test_helper(":test!test@test PRIVMSG test :IDENTIFY\r\n", |_| {});
        assert_eq!(data[], "NOTICE test :Commands must be prefixed by CS or NS.\r\n")
    }

    #[test]
    fn non_command_message_in_channel() {
        let (data, _) = test_helper(":test!test@test PRIVMSG #test :Hi there!\r\n", |_| {});
        assert_eq!(data[], "");
    }

    #[test]
    fn non_command_message_in_query() {
        let (data, _) = test_helper(":test!test@test PRIVMSG test :CS line\r\n", |_| {});
        assert_eq!(data[], "NOTICE test :line is not a valid command.\r\n");
    }

    #[test]
    fn owner_on_join() {
        let mut ch = Channel::new("#test11", "test", "test").unwrap();
        ch.admins.push("test".into_string());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(":test!test@test JOIN :#test11\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(data[], "SAMODE #test11 +qa test\r\n");
    }

    #[test]
    fn admin_on_join() {
        let mut ch = Channel::new("#test8", "test", "owner").unwrap();
        ch.admins.push("test".into_string());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(":test!test@test JOIN :#test8\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(data[], "SAMODE #test8 +a test\r\n");
    }

    #[test]
    fn oper_on_join() {
        let mut ch = Channel::new("#test9", "test", "owner").unwrap();
        ch.opers.push("test".into_string());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(":test!test@test JOIN :#test9\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(data[], "SAMODE #test9 +o test\r\n");
    }

    #[test]
    fn voice_on_join() {
        let mut ch = Channel::new("#test10", "test", "owner").unwrap();
        ch.voice.push("test".into_string());
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(":test!test@test JOIN :#test10\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(data[], "SAMODE #test10 +v test\r\n");
    }

    #[test]
    fn unidentify_on_quit() {
        let (data, state) = test_helper(":test!test@test QUIT :Goodbye!\r\n", |state| {
            state.identify("test");
        });
        assert!(state.no_users_identified())
        assert_eq!(data[], "");
    }

    #[test]
    fn update_topic() {
        let ch = Channel::new("#test23", "test", "owner").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(":test!test@test TOPIC #test23 :This is a topic.\r\n", |_| {});
        assert_eq!(data[], "");
        let ch = Channel::load("#test23").unwrap();
        assert_eq!(ch.topic[], "This is a topic.");
    }

    #[cfg(not(feature = "democracy"))]
    #[test]
    fn voicing_identified_user() {
        let (data, _) = test_helper(":test!test@test MODE #test +v test\r\n", |state| {
            state.identify("test");
        });
        assert_eq!(data[], "");
    }
    
    #[cfg(feature = "democracy")]
    #[test]
    fn voicing_identified_user() {
        let ch = Channel::new("#test26", "test", "owner").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(":test!test@test MODE #test26 +v test\r\n", |state| {
            state.identify("test");
        });
        let ch = Channel::load("#test26").unwrap();
        assert_eq!(ch.voice, vec!["test".into_string()]);
        assert_eq!(data[], "");
    }
    
    #[cfg(not(feature = "democracy"))]
    #[test]
    fn voicing_unidentified_user() {
        let (data, _) = test_helper(":test!test@test MODE #test +v test\r\n", |_| {});
        assert_eq!(data[], "");
    }

    #[cfg(feature = "democracy")]
    #[test]
    fn voicing_unidentified_user() {
        let ch = Channel::new("#test27", "test", "owner").unwrap();
        assert!(ch.save().is_ok());
        let (data, _) = test_helper(":test!test@test MODE #test27 +v test\r\n", |_| {});
        let ch = Channel::load("#test27").unwrap();
        assert!(ch.voice.is_empty());
        assert_eq!(data[], "SAMODE #test27 -v test\r\n");
    }

    #[cfg(not(feature = "democracy"))]
    #[test]
    fn voicing_user_on_unregistered_channel() {
        let (data, _) = test_helper(":test!test@test MODE #unregistered +v test\r\n", |_| {});
        assert_eq!(data[], "");
    }

    #[cfg(feature = "democracy")]
    #[test]
    fn voicing_user_on_unregistered_channel() {
        let (data, _) = test_helper(":test!test@test MODE #unregistered +v test\r\n", |_| {});
        assert_eq!(data[], "");
    }

    #[test]
    fn upper_case() {
        assert_eq!(super::upper_case("identify")[], "IDENTIFY")
    }

    #[cfg(feature = "derp")]
    #[test]
    fn derp_test() {
        let _ = unlink(&Path::new("data/derp.json"));
        let (data, _) = test_helper(":test!test@test PRIVMSG test :!derp\r\n", |_| {});
        assert_eq!(data[], "PRIVMSG test :There have been 0 derps.\r\n");
        let (data, _) = test_helper(":test!test@test PRIVMSG #test :!derp++\r\n", |_| {});
        assert_eq!(data[], "PRIVMSG #test :There has been 1 derp.\r\n");
    }
}
