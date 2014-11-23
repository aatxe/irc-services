#[cfg(feature = "democracy")] use std::collections::HashMap;
#[cfg(feature = "resistance")] #[cfg(not(feature = "democracy"))] use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(feature = "democracy")] use std::sync::MutexGuard;
#[cfg(feature = "resistance")] #[cfg(not(feature = "democracy"))] use std::sync::MutexGuard;
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
        self.identified.lock().push(nick.into_string())
    }

    pub fn is_identified(&self, nick: &str) -> bool {
        self.identified.lock().contains(&nick.into_string())
    }

    pub fn remove(&self, nick: &str) {
        let mut identified = self.identified.lock();
        if let Some(i) = identified.position_elem(&nick.into_string()) {
            identified.swap_remove(i);
        }
    }

    #[cfg(test)]
    pub fn no_users_identified(&self) -> bool {
        self.identified.lock().is_empty()
    }

    #[cfg(feature = "resistance")]
    pub fn get_games<'a>(&'a self) -> MutexGuard<'a, HashMap<String, Resistance>> {
        self.resistance.lock()
    }

    #[cfg(feature = "democracy")]
    pub fn get_votes<'a>(&'a self) -> MutexGuard<'a, HashMap<String, Democracy>> {
        self.democracy.lock()
    }

    #[cfg(feature = "democracy")]
    pub fn get_online_voting_pop(&self, chan: &str) -> uint {
        if let Ok(mut chan) = Channel::load(chan) {
            chan.voice.retain(|u| self.identified.lock().contains(u));
            chan.voice.len()
        } else {
            0u
        }
    }

    #[cfg(feature = "democracy")]
    pub fn get_voting_pop(&self, chan: &str) -> uint {
        if let Ok(chan) = Channel::load(chan) {
            chan.voice.len()
        } else {
            0u
        }
    }
}
