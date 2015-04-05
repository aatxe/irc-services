use std::borrow::ToOwned;
#[cfg(any(feature = "democracy", feature = "resistance"))] use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(any(feature = "democracy", feature = "resistance"))] use std::sync::MutexGuard;
#[cfg(feature = "democracy")] use data::channel::Channel;
#[cfg(feature = "democracy")] use data::democracy::Democracy;
#[cfg(feature = "resistance")] use data::resistance::Resistance;

pub struct State {
    identified: Mutex<Vec<String>>,
    #[cfg(feature = "resistance")]
    resistance: Mutex<HashMap<String, Resistance>>,
    #[cfg(feature = "democracy")]
    democracy: Mutex<HashMap<String, Democracy>>,
}

impl State {
    #[cfg(not(feature = "resistance"))]
    #[cfg(not(feature = "democracy"))]
    pub fn new() -> State {
        State { identified: Mutex::new(Vec::new()) }
    }

    #[cfg(not(feature = "democracy"))]
    #[cfg(feature = "resistance")]
    pub fn new() -> State {
        State { identified: Mutex::new(Vec::new()), resistance: Mutex::new(HashMap::new()) }
    }

    #[cfg(not(feature = "resistance"))]
    #[cfg(feature = "democracy")]
    pub fn new() -> State {
        State { identified: Mutex::new(Vec::new()), democracy: Mutex::new(HashMap::new()) }
    }

    #[cfg(feature = "resistance")]
    #[cfg(feature = "democracy")]
    pub fn new() -> State {
        State { 
            identified: Mutex::new(Vec::new()),
            resistance: Mutex::new(HashMap::new()),
            democracy:  Mutex::new(HashMap::new())
        }
    }

    pub fn identify(&self, nick: &str) {
        self.identified.lock().unwrap().push(nick.to_owned())
    }

    pub fn is_identified(&self, nick: &str) -> bool {
        self.identified.lock().unwrap().contains(&nick.to_owned())
    }

    pub fn remove(&self, nick: &str) {
        let mut identified = self.identified.lock().unwrap();
        if let Some(i) = identified.iter().position(|n| &n[..] == nick) {
            identified.swap_remove(i);
        }
    }

    #[cfg(test)]
    pub fn no_users_identified(&self) -> bool {
        self.identified.lock().unwrap().is_empty()
    }

    #[cfg(feature = "resistance")]
    pub fn get_games<'a>(&'a self) -> MutexGuard<'a, HashMap<String, Resistance>> {
        self.resistance.lock().unwrap()
    }

    #[cfg(feature = "democracy")]
    pub fn get_votes<'a>(&'a self) -> MutexGuard<'a, HashMap<String, Democracy>> {
        self.democracy.lock().unwrap()
    }

    #[cfg(feature = "democracy")]
    pub fn get_online_voting_pop(&self, chan: &str) -> usize {
        if let Ok(mut chan) = Channel::load(chan) {
            chan.voice.retain(|u| self.identified.lock().unwrap().contains(u));
            chan.voice.len()
        } else {
            0
        }
    }

    #[cfg(feature = "democracy")]
    pub fn get_voting_pop(&self, chan: &str) -> usize {
        if let Ok(chan) = Channel::load(chan) {
            chan.voice.len()
        } else {
            0
        }
    }

    #[cfg(feature = "democracy")]
    pub fn is_voiced(&self, user: &str, chan: &str) -> bool {
        if let Ok(chan) = Channel::load(chan) {
            chan.voice.contains(&user.to_owned())
        } else {
            false
        }
    }
}
