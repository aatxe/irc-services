#![cfg(feature = "resistance")]
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::old_io::IoResult;
use std::num::Float;
use std::rand::{Rng, ThreadRng, thread_rng};
use irc::client::data::kinds::{IrcReader, IrcWriter};
use irc::client::server::Server;
use irc::client::server::utils::Wrapper;

pub struct Resistance {
    chan: String,
    started: bool,
    rng: ThreadRng,
    players: Vec<String>,
    rebels: Vec<String>,
    spies: Vec<String>,
    missions_won: u8,
    missions_run: u8,
    rejected_proposals: u8,
    // Mission-specific stuffs.
    leader: String, proposed_members: Vec<String>,
    votes_for_mission: HashMap<String, Vote>,
    mission_votes: HashMap<String, Vote>,
}

#[derive(Clone, PartialEq)]
enum Vote {
    Success,
    Failure,
    NotYetVoted,
}

impl Resistance {
    pub fn new_game(user: &str, chan: &str) -> Resistance {
        Resistance {
            chan: chan.to_owned(), started: false, rng: thread_rng(),
            players: vec![user.to_owned()], rebels: Vec::new(), spies: Vec::new(),
            missions_won: 0u8, missions_run: 0u8, rejected_proposals: 0u8,
            leader: user.to_owned(), proposed_members: Vec::new(),
            votes_for_mission: HashMap::new(), mission_votes: HashMap::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.missions_run == 5 || self.missions_won == 3
        || self.rejected_proposals == 5 || (self.missions_run - self.missions_won >= 3)
    }

    pub fn is_leader(&self, nick: &str) -> bool {
        &self.leader[] == nick
    }

    pub fn start<'a, T: IrcReader, U: IrcWriter>(&mut self, server: &'a Wrapper<'a, T, U>) 
        -> IoResult<()> {
        if self.started {
            server.send_privmsg(&self.chan[], "The game has already begun!")
        } else if self.total_players() > 4 {
            self.started = true;
            self.rng.shuffle(self.players.as_mut_slice());
            for user in self.players.clone().iter() {
                if self.spies.len() < (self.total_players() as f32 * 0.4).round() as usize {
                    try!(self.add_spy(server, &user[]));
                } else {
                    try!(self.add_rebel(server, &user[]));
                }
            }
            for user in self.spies.iter() {
                try!(server.send_privmsg(&user[], &format!("Spies: {:?}", self.spies)[]));
            }
            try!(server.send_privmsg(&self.chan[], "The game has begun!"));
            server.send_privmsg(&self.chan[], &format!("The first mission requires {} participants\
                                .",self.get_number_for_next_mission())[])
        } else {
            server.send_privmsg(&self.chan[], "You need at least five players to play.")
        }
    }

    pub fn add_player<'a, T: IrcReader, U: IrcWriter>(&mut self, server: &'a Wrapper<'a, T, U>, 
                                                      nick: &str) -> IoResult<()> {
        if self.started {
            try!(server.send_privmsg(&self.chan[], "Sorry, the game is already in progress!"));
        } else if self.players.contains(&nick.to_owned()) {
            try!(server.send_privmsg(&self.chan[], "You've already joined this game!"));
        } else if self.total_players() < 10 {
            self.players.push(nick.to_owned());
            try!(server.send_privmsg(nick, "You've joined the game. You'll get your position when \
                                            it starts."));
        } else {
            try!(server.send_privmsg(&self.chan[], "Sorry, the game is full!"));
        }
        Ok(())
    }

    pub fn propose_mission<'a, T: IrcReader, U: IrcWriter>(&mut self, server: &'a Wrapper<'a, T, U>
                                                           , user: &str, users: &str) 
        -> IoResult<()> {
        if !self.proposed_members.is_empty() || user != &self.leader[] || !self.started {
            return Ok(())
        }
        let mut users: Vec<_> = users.split_str(" ").collect();
        users.retain(|user| user.len() != 0);
        let valid = try!(if self.total_players() > 7 {
            self.validate_mission(server, users.len(), 3, 4, 4, 5, 5)
        } else if self.total_players() == 7 {
            self.validate_mission(server, users.len(), 2, 3, 3, 4, 4)
        } else if self.total_players() == 6 {
            self.validate_mission(server, users.len(), 2, 3, 4, 3, 4)
        } else {
            self.validate_mission(server, users.len(), 2, 3, 2, 3, 3)
        });
        if users.iter().cloned().partition::<Vec<_>, _>(|&user| {
                self.players.contains(&user.to_owned())
            }).1.len() != 0 {
            try!(
                server.send_privmsg(&self.chan[], "Proposals must only include registered players.")
            );
        } else if valid {
            for user in users.iter() {
                self.proposed_members.push((*user).to_owned());
            }
            for user in self.rebels.iter().chain(self.spies.iter()) {
                self.votes_for_mission.insert(user.clone(), Vote::NotYetVoted);
            }
            try!(server.send_privmsg(&self.chan[], &format!("Proposed mission: {:?}",
                                     self.proposed_members)[]));
        }
        Ok(())
    }

    pub fn cast_proposal_vote<'a, T: IrcReader, U: IrcWriter>(&mut self, 
                                                              server: &'a Wrapper<'a, T, U>, 
                                                              user: &str, vote: &str)
        -> IoResult<()> {
        if !self.players.contains(&user.to_owned()) {
            try!(server.send_privmsg(user, "You're not involved in this game."));
            return Ok(())
        } else if self.proposed_members.is_empty() {
            try!(server.send_privmsg(user, "There is no current mission proposal."));
            return Ok(())
        } else if vote.starts_with("y") || vote.starts_with("Y") {
            self.votes_for_mission.insert(user.to_owned(), Vote::Success);
            try!(server.send_privmsg(&self.chan[], "A vote has been cast."));
        } else if vote.starts_with("n") || vote.starts_with("N") {
            self.votes_for_mission.insert(user.to_owned(), Vote::Failure);
            try!(server.send_privmsg(&self.chan[], "A vote has been cast."));
        } else {
            try!(server.send_privmsg(&self.chan[], "You must vote yea or nay."));
            return Ok(());
        }
        let result = self.get_proposal_result();
        if result != Vote::NotYetVoted {
            if result == Vote::Success {
                try!(self.run_mission(server));
            } else {
                self.get_new_leader();
                self.rejected_proposals += 1;
                self.proposed_members = Vec::new();
                try!(server.send_privmsg(&self.chan[], &format!("The proposal was rejected ({} / 5\
                    ). The new leader is {}.", self.rejected_proposals, self.leader
                )[]));
            }
        }
        Ok(())
    }

    pub fn cast_mission_vote<'a, T: IrcReader, U: IrcWriter>(&mut self, 
                                                             server: &'a Wrapper<'a, T, U>, 
                                                             user: &str, vote: &str)
        -> IoResult<()> {
        if !self.players.contains(&user.to_owned()) {
            try!(server.send_privmsg(user, "You're not involved in this game."));
            return Ok(())
        } else if self.mission_votes.is_empty() {
            try!(server.send_privmsg(user, "There is no mission in progress."));
            return Ok(());
        } else if !self.mission_votes.contains_key(&user.to_owned()) {
            try!(server.send_privmsg(user, "You're not involved in this mission."));
            return Ok(());
        } else if vote.starts_with("y") || vote.starts_with("Y") {
            self.mission_votes.insert(user.to_owned(), Vote::Success);
            try!(server.send_privmsg(user, "Your vote has been cast."));
        } else if vote.starts_with("n") || vote.starts_with("N") {
            self.mission_votes.insert(user.to_owned(), Vote::Failure);
            try!(server.send_privmsg(user, "Your vote has been cast."));
        } else {
            try!(server.send_privmsg(user, "You must vote yea or nay."));
            return Ok(());
        }
        let (result, fails) = self.get_mission_result();
        if result != Vote::NotYetVoted {
            self.get_new_leader();
            self.missions_run += 1;
            if result == Vote::Success {
                self.missions_won += 1;
                try!(server.send_privmsg(&self.chan[], &format!("The mission was a success (S: {} \
                    / {}). The new leader is {}. The next mission requires {} participants.",
                    self.missions_won, self.missions_run, self.leader,
                    self.get_number_for_next_mission()
                )[]));
            } else {
                try!(server.send_privmsg(&self.chan[], &format!("The mission was a failure with {\
                    } saboteurs (S: {} / {}). The new leader is {}. The next mission requires {} \
                    participants.", fails, self.missions_won, self.missions_run, self.leader,
                    self.get_number_for_next_mission()
                )[]));
            }
            self.mission_votes = HashMap::new();
            if self.is_complete() {
                if self.missions_won == 3 {
                    try!(server.send_privmsg(&self.chan[], "Game over: Rebels win!"));
                } else {
                    try!(server.send_privmsg(&self.chan[], "Game over: Spies win!"));
                }
            }
        }
        Ok(())
    }

    pub fn list_players<'a, T: IrcReader, U: IrcWriter>(&self, server: &'a Wrapper<'a, T, U>) 
        -> IoResult<()> {
        server.send_privmsg(&self.chan[], &format!("Players: {:?}", self.players)[])
    }

    fn add_rebel<'a, T: IrcReader, U: IrcWriter>(&mut self, server: &'a Wrapper<'a, T, U>, 
                                                 nick: &str) -> IoResult<()> {
        self.rebels.push(nick.to_owned());
        server.send_privmsg(nick, &format!("You're a rebel in {}.", self.chan)[])
    }

    fn add_spy<'a, T: IrcReader, U: IrcWriter>(&mut self, server: &'a Wrapper<'a, T, U>, 
                                               nick: &str) -> IoResult<()> {
        self.spies.push(nick.to_owned());
        server.send_privmsg(nick, &format!("You're a spy in {}.", self.chan)[])
    }

    fn total_players(&self) -> usize {
        self.players.len()
    }

    fn get_mission_result(&self) -> (Vote, u8) {
        let special = self.missions_run == 4 && self.total_players() > 6;
        let mut fails = 0u8;
        for vote in self.mission_votes.values() {
            if vote == &Vote::NotYetVoted {
                return (Vote::NotYetVoted, 0)
            } else if vote == &Vote::Failure {
                fails += 1;
            }
        }
        (if fails == 0 || special && fails == 1 { Vote::Success } else { Vote::Failure }, fails)
    }

    fn get_proposal_result(&self) -> Vote {
        let mut yea = 0u8;
        let mut nay = 0u8;
        for vote in self.votes_for_mission.values() {
            if vote == &Vote::Success {
                yea += 1;
            } else if vote == &Vote::Failure {
                nay += 1;
            } else {
                return Vote::NotYetVoted
            }
        }
        if yea > nay { Vote::Success } else { Vote::Failure }
    }

    fn get_new_leader(&mut self) {
        self.rng.shuffle(self.players.as_mut_slice());
        if self.is_leader(&self.players[0][]) {
            self.leader = self.players[1].clone();
        } else {
            self.leader = self.players[0].clone();
        }
    }

    fn validate_mission<'a, T: IrcReader, U: IrcWriter>(&self, server: &'a Wrapper<'a, T, U>, 
                                                        len: usize, m1: usize, m2: usize, 
                                                        m3: usize, m4: usize, m5: usize) 
    -> IoResult<bool> {
        match self.missions_run {
            0 => if len != m1 {
                try!(server.send_privmsg(&self.chan[],
                                         &format!("Mission 1 should have {} members.", m1)[]));
                return Ok(false)
            },
            1 => if len != m2 {
                try!(server.send_privmsg(&self.chan[],
                                         &format!("Mission 2 should have {} members.", m2)[]));
                return Ok(false)
            },
            2 => if len != m3 {
                try!(server.send_privmsg(&self.chan[],
                                         &format!("Mission 3 should have {} members.", m3)[]));
                return Ok(false)
            },
            3 => if len != m4 {
                try!(server.send_privmsg(&self.chan[],
                                         &format!("Mission 4 should have {} members.", m4)[]));
                return Ok(false)
            },
            4 => if len != m5 {
                try!(server.send_privmsg(&self.chan[],
                                         &format!("Mission 5 should have {} members.", m5)[]));
                return Ok(false)
            },
            _ => {
                try!(server.send_privmsg(&self.chan[], "Something went wrong."));
                return Ok(false)
            }
        }
        Ok(true)
    }

    fn get_number_for_next_mission(&self) -> usize {
        if self.total_players() > 7 {
            self.mission_number_helper(3, 4, 4, 5, 5)
        } else if self.total_players() == 7 {
            self.mission_number_helper(2, 3, 3, 4, 4)
        } else if self.total_players() == 6 {
            self.mission_number_helper(2, 3, 4, 3, 4)
        } else {
            self.mission_number_helper(2, 3, 2, 3, 3)
        }
    }

    fn mission_number_helper(&self, m1: usize, m2: usize, m3: usize, m4: usize, m5: usize) 
    -> usize {
        match self.missions_run {
            0 => m1,
            1 => m2,
            2 => m3,
            3 => m4,
            5 => m5,
            _ => 0,
        }
    }

    fn run_mission<'a, T: IrcReader, U: IrcWriter>(&mut self, server: &'a Wrapper<'a, T, U>) 
    -> IoResult<()> {
        for user in self.proposed_members.iter() {
            self.mission_votes.insert(user.clone(), Vote::NotYetVoted);
        }
        self.proposed_members = Vec::new();
        self.rejected_proposals = 0;
        server.send_privmsg(&self.chan[], "The mission is now live!")
    }
}
