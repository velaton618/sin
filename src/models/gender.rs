use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Gender {
    Male,
    Female,
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Gender::Male => write!(f, "Male"),
            Gender::Female => write!(f, "Female"),
        }
    }
}

impl Gender {
    pub fn from_str(s: &str) -> Result<Gender, &'static str> {
        match s.to_lowercase().as_str() {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            _ => Err("Invalid gender string"),
        }
    }
}

impl From<i32> for Gender {
    fn from(value: i32) -> Self {
        match value {
            1 => Gender::Female,
            _ => Gender::Male,
        }
    }
}
