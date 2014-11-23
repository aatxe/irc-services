#[cfg(feature = "resistance")] use std::collections::HashMap;
use std::sync::Mutex;
#[cfg(feature = "resistance")] use std::sync::MutexGuard;
#[cfg(feature = "resistance")] use data::resistance::Resistance;

pub struct State {
    identified: Mutex<Vec<String>>,
    #[cfg(feature = "resistance")]
    resistance: Mutex<HashMap<String, Resistance>>,
}

impl State {
    #[cfg(not(feature = "resistance"))]
    pub fn new() -> State {
        State { identified: Mutex::new(Vec::new()) }
    }

    #[cfg(feature = "resistance")]
    pub fn new() -> State {
        State { identified: Mutex::new(Vec::new()), resistance: Mutex::new(HashMap::new()) }
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
}
