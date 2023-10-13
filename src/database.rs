use crate::models::gender::Gender;
use crate::models::user::User;
use rusqlite::{params, Connection, Result};

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let connection = Connection::open(db_path)?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS users (
                  id INTEGER PRIMARY KEY,
                  nickname TEXT NOT NULL,
                  age INTEGER NOT NULL,
                  gender TEXT NOT NULL,
                  state TEXT NOT NULL
                  )",
            [],
        )?;
        Ok(Database { connection })
    }

    pub fn add_user(&self, user: &User) -> Result<()> {
        self.connection.execute(
            "INSERT INTO users (nickname, age, gender) VALUES (?1, ?2, ?3)",
            params![user.nickname.clone(), user.age, user.gender.to_string()],
        )?;
        Ok(())
    }

    pub fn get_all_users(&self) -> Result<Vec<User>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, nickname, age, gender FROM users")?;
        let user_iter = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                nickname: row.get(1)?,
                age: row.get(2)?,
                gender: match row.get::<_, String>(3)?.as_str() {
                    "Male" => Gender::Male,
                    "Female" => Gender::Female,
                    _ => unreachable!(),
                },
            })
        })?;

        let mut users = Vec::new();
        for user in user_iter {
            users.push(user?);
        }
        Ok(users)
    }
}
