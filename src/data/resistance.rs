#![cfg(feature = "resistance")]
use std::collections::HashMap;
use std::io::IoResult;
use std::rand::random;
use irc::data::kinds::IrcStream;
use irc::server::Server;
use irc::server::utils::Wrapper;

pub struct Resistance<T> where T: IrcStream {
    chan: String,
    started: bool,
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

#[deriving(Clone, PartialEq)]
enum Vote {
    Success,
    Failure,
    NotYetVoted,
}

impl<'a, T> Resistance<T> where T: IrcStream {
    pub fn new_game(user: &str, chan: &str) -> Resistance<T> {
        Resistance {
            chan: chan.into_string(), started: false,
            rebels: Vec::new(), spies: Vec::new(),
            missions_won: 0u8, missions_run: 0u8, rejected_proposals: 0u8,
            leader: user.into_string(), proposed_members: Vec::new(),
            votes_for_mission: HashMap::new(), mission_votes: HashMap::new(),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.missions_run == 5 || self.missions_won == 3 || self.rejected_proposals == 5
    }

    pub fn start(&mut self, server: &'a Wrapper<'a, T>) -> IoResult<()> {
        if self.total_players() > 4 {
            self.started = true;
            server.send_privmsg(self.chan[], "The game has begun!")
        } else {
            server.send_privmsg(self.chan[], "You need at least five players to play.")
        }
    }

    pub fn add_player(&mut self, server: &'a Wrapper<'a, T>, nick: &str) -> IoResult<()> {
        if self.started {
            try!(server.send_privmsg(self.chan[], "Sorry, the game is already in progress!"));
        } else if random::<bool>() && self.spies.len() < (self.total_players() * 2) / 5 {
            try!(self.add_spy(server, nick));
        } else if self.total_players() < 10 {
            try!(self.add_rebel(server, nick));
        } else {
            try!(server.send_privmsg(self.chan[], "Sorry, the game is full!"));
        }
        Ok(())
    }

    pub fn propose_mission(&mut self, server: &'a Wrapper<'a, T>, users: &str) -> IoResult<()> {
        let users: Vec<_> = users.split_str(" ").collect();
        let valid = try!(if self.total_players() > 7 {
            self.validate_mission(server, users.len(), 3, 4, 4, 5, 5)
        } else if self.total_players() == 7 {
            self.validate_mission(server, users.len(), 2, 3, 3, 4, 4)
        } else if self.total_players() == 6 {
            self.validate_mission(server, users.len(), 2, 3, 2, 3, 3)
        } else {
            self.validate_mission(server, users.len(), 3, 4, 4, 5, 5)
        });
        if valid {
            for user in users.into_iter() {
                self.proposed_members.push(user.into_string());
            }
            try!(server.send_privmsg(self.chan[],
                format!("Proposed mission: {}", self.proposed_members)[]));
        }
        Ok(())
    }

    pub fn cast_proposal_vote(&mut self, server: &'a Wrapper<'a, T>, user: &str, vote: &str) -> IoResult<()> {
        if self.proposed_members.is_empty() {
            try!(server.send_privmsg(user, "There is no current mission proposal."));
        } else if vote == "yea" || vote == "YEA" || vote == "Yea" {
            self.votes_for_mission.insert(user.into_string(), Success);
            try!(server.send_privmsg(self.chan[], "A vote has been cast."));
        } else if vote == "nay" || vote == "NAY" || vote == "Nay" {
            self.votes_for_mission.insert(user.into_string(), Failure);
            try!(server.send_privmsg(self.chan[], "A vote has been cast."));
        } else {
            try!(server.send_privmsg(self.chan[], "You must vote yea or nay."));
            return Ok(());
        }
        let result = self.get_proposal_result();
        if result != NotYetVoted {
            self.get_new_leader();
            self.missions_run += 1;
            if result == Success {
                try!(self.run_mission(server));
            } else {
                try!(server.send_privmsg(self.chan[],
                     format!("The proposal was rejected ({} / 5). The new leader is {}.",
                             self.rejected_proposals, self.leader)[]
                ));
            }
        }
        Ok(())
    }

    pub fn cast_mission_vote(&mut self, server: &'a Wrapper<'a, T>, user: &str, vote: &str) -> IoResult<()> {
        if self.mission_votes.is_empty() {
            try!(server.send_privmsg(user, "There is no mission in progress."));
        } else if !self.mission_votes.contains_key(&user.into_string()) {
            try!(server.send_privmsg(user, "You're not involved in this mission."));
        } else if vote == "yea" || vote == "YEA" || vote == "Yea" {
            self.mission_votes.insert(user.into_string(), Success);
            try!(server.send_privmsg(user, "Your vote has been cast."));
        } else if vote == "nay" || vote == "NAY" || vote == "Nay" {
            self.mission_votes.insert(user.into_string(), Failure);
            try!(server.send_privmsg(user, "Your vote has been cast."));
        } else {
            try!(server.send_privmsg(user, "You must vote yea or nay."));
            return Ok(());
        }
        let (result, fails) = self.get_mission_result();
        if result != NotYetVoted {
            if result == Success {
                self.missions_won += 1;
                try!(server.send_privmsg(self.chan[],
                     format!("The mission was a success (S: {} / {}). The new leader is {}.",
                             self.missions_won, self.missions_run, self.leader)[]
                ));
            } else {
                try!(server.send_privmsg(self.chan[],
                     format!("The mission was a failure with {} saboteurs (S: {} / {}). The new leader is {}.",
                             fails, self.missions_won, self.missions_run, self.leader)[]
                ));
            }
            if self.is_complete() {
                if self.missions_won == 3 {
                    try!(server.send_privmsg(self.chan[], "Game over: Rebels win!"));
                } else {
                    try!(server.send_privmsg(self.chan[], "Game over: Spies win!"));
                }
            }
        }
        Ok(())
    }

    fn add_rebel(&mut self, server: &'a Wrapper<'a, T>, nick: &str) -> IoResult<()> {
        self.rebels.push(nick.into_string());
        server.send_privmsg(nick, format!("You're a rebel in {}.", self.chan)[])
    }

    fn add_spy(&mut self, server: &'a Wrapper<'a, T>, nick: &str) -> IoResult<()> {
        self.spies.push(nick.into_string());
        server.send_privmsg(nick, format!("You're a spy in {}.", self.chan)[])
    }

    fn total_players(&self) -> uint {
        self.rebels.len() + self.spies.len()
    }

    fn get_mission_result(&self) -> (Vote, u8) {
        let special = self.missions_run == 4 && self.total_players() > 6;
        let mut fails = 0u8;
        for vote in self.mission_votes.values() {
            if vote == &NotYetVoted {
                return (NotYetVoted, 0)
            } else if vote == &Failure {
                fails += 1;
            }
        }
        (if fails == 0 || special && fails == 1 { Success } else { Failure }, fails)
    }

    fn get_proposal_result(&self) -> Vote {
        let mut yea = 0u8;
        let mut nay = 0u8;
        for vote in self.votes_for_mission.values() {
            if vote == &Success {
                yea += 1;
            } else if vote == &Failure {
                nay += 1;
            } else {
                return NotYetVoted
            }
        }
        if yea > nay { Success } else { Failure }
    }

    fn get_new_leader(&mut self) {

    }

    fn validate_mission(&self, server: &'a Wrapper<'a, T>, len: uint, m1: uint, m2: uint, m3: uint,
                        m4: uint, m5: uint) -> IoResult<bool> {
        match self.missions_run {
            0 => if len != m1 {
                try!(server.send_privmsg(self.chan[],
                                              format!("Mission 1 should have {} members.", m1)[]));
                return Ok(false)
            },
            1 => if len != m2 {
                try!(server.send_privmsg(self.chan[],
                                              format!("Mission 2 should have {} members.", m2)[]));
                return Ok(false)
            },
            2 => if len != m3 {
                try!(server.send_privmsg(self.chan[],
                                              format!("Mission 3 should have {} members.", m3)[]));
                return Ok(false)
            },
            3 => if len != m4 {
                try!(server.send_privmsg(self.chan[],
                                              format!("Mission 4 should have {} members.", m4)[]));
                return Ok(false)
            },
            4 => if len != m5 {
                try!(server.send_privmsg(self.chan[],
                                              format!("Mission 5 should have {} members.", m5)[]));
                return Ok(false)
            },
            _ => {
                try!(server.send_privmsg(self.chan[], "Something went wrong."));
                return Ok(false)
            }
        }
        Ok(true)
    }

    fn run_mission(&mut self, server: &'a Wrapper<'a, T>) -> IoResult<()> {
        for user in self.proposed_members.iter() {
            self.mission_votes.insert(user.clone(), NotYetVoted);
        }
        self.proposed_members = Vec::new();
        self.rejected_proposals = 0;
        server.send_privmsg(self.chan[], "The mission is now live!")
    }
}
