use std::cell::RefCell;

pub struct State {
    identified: RefCell<Vec<String>>
}

impl State {
    pub fn new() -> State {
        State { identified: RefCell::new(Vec::new()) }
    }

    pub fn identify(&self, nick: &str) {
        self.identified.borrow_mut().push(nick.into_string())
    }

    pub fn is_identified(&self, nick: &str) -> bool {
        self.identified.borrow().contains(&nick.into_string())
    }

    pub fn remove(&self, nick: &str) {
        if let Some(i) = self.identified.borrow().position_elem(&nick.into_string()) {
            self.identified.borrow_mut().swap_remove(i);
        }
    }
}
