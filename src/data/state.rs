use std::sync::Mutex;

pub struct State {
    identified: Mutex<Vec<String>>
}

impl State {
    pub fn new() -> State {
        State { identified: Mutex::new(Vec::new()) }
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
}
