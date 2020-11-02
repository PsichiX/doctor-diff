use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct HashValue(pub Vec<u8>);

impl ToString for HashValue {
    fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.0.len() * 2);
        for item in &self.0 {
            result.push_str(&format!("{:02x}", item));
        }
        result
    }
}

impl std::fmt::Debug for HashValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}
