use std::collections::HashMap;

// TODO: Do something less stupid here.
pub struct Symbols {
    pub to_symbol: HashMap<String, u32>,
    pub to_name: Vec<String>,
}

impl Symbols {
    pub fn new() -> Self {
        return Self {
            to_symbol: HashMap::new(),
            to_name: Vec::new(),
        };
    }

    pub fn add_str<T: AsRef<str> + Into<String>>(&mut self, s: T) -> u32 {
        if let Some(id) = self.to_symbol.get(s.as_ref()) {
            return *id;
        }

        let id = self.to_name.len() as u32 + 1;
        let s = s.into();
        self.to_symbol.insert(s.clone(), id);
        self.to_name.push(s);
        return id;
    }

    pub fn from_str(&self, s: &str) -> Option<u32> {
        if let Some(id) = self.to_symbol.get(s) {
            return Some((*id).into());
        }

        return None;
    }

    pub fn to_str(&self, id: u32) -> Option<&str> {
        return self.to_name.get(id as usize).map(|a| -> &str { &*a });
    }
}
