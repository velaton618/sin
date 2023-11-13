use crate::{
    models::{gender::Gender, user::User},
    user_state::UserState,
};
use rusqlite::{params, Connection, OptionalExtension, Result};

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
                  state INTEGER DEFAULT 0
                  )",
            [],
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS queue (
                  user_id INTEGER PRIMARY KEY,
                  search_gender INTEGER DEFAULT 0,
                  searcher_gender INTEGER NOT NULL
                  )",
            [],
        )?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS chats (
                  id INTEGER PRIMARY KEY,
                  chat_one INTEGER KEY NOT NULL,
                  chat_two INTEGER KEY NOT NULL
                  )",
            [],
        )?;

        Ok(Database { connection })
    }

    pub fn create_chat(&self, user_id_one: i64, user_id_two: i64) -> Result<()> {
        self.connection.execute(
            "INSERT INTO chats (chat_one, chat_two) VALUES (?1, ?2)",
            params![user_id_one, user_id_two],
        )?;
        Ok(())
    }

    pub fn get_chat(&self, user_id: i64) -> Result<Option<i64>> {
        let mut stmt = self
            .connection
            .prepare("SELECT chat_one, chat_two FROM chats WHERE chat_one = ?1 OR chat_two = ?1")?;

        let chat = stmt
            .query_row(params![user_id], |row| {
                let chat_one: i64 = row.get(0)?;
                let chat_two: i64 = row.get(1)?;
                if chat_one == user_id {
                    Ok(chat_two)
                } else {
                    Ok(chat_one)
                }
            })
            .optional()?;

        Ok(chat)
    }

    pub fn add_user(&self, user: &User) -> Result<()> {
        self.connection.execute(
            "INSERT INTO users (id, nickname, age, gender) VALUES (?1, ?2, ?3, ?4)",
            params![
                user.id.clone(),
                user.nickname.clone(),
                user.age,
                user.gender.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn get_user(&self, user_id: i64) -> Result<Option<User>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, nickname, age, gender, state FROM users WHERE id = ?1")?;
        let user = stmt
            .query_row(params![user_id], |row| {
                let id: i64 = row.get(0)?;
                let nickname: String = row.get(1)?;
                let age: u8 = row.get(2)?;
                let gender_str: String = row.get(3)?;
                let gender = match Gender::from_str(&gender_str) {
                    Ok(g) => g,
                    Err(_) => Gender::Male,
                };
                let state_int: i32 = row.get(4)?;
                let state = UserState::from(state_int);

                Ok(User {
                    id: id,
                    nickname,
                    age,
                    gender,
                    state,
                })
            })
            .optional()?;
        Ok(user)
    }

    pub fn get_all_users(&self) -> Result<Vec<User>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, nickname, age, gender, state FROM users")?;

        let user_iter = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let nickname: String = row.get(1)?;
            let age: u8 = row.get(2)?;
            let gender_str: String = row.get(3)?;
            let gender = match Gender::from_str(&gender_str) {
                Ok(g) => g,
                Err(_) => Gender::Male,
            };
            let state_int: i32 = row.get(4)?;
            let state = UserState::from(state_int);

            Ok(User {
                id: id,
                nickname,
                age,
                gender,
                state,
            })
        })?;

        let users: Result<Vec<User>> = user_iter.collect();
        Ok(users?)
    }

    pub fn get_user_state(&self, user_id: i64) -> Result<Option<UserState>> {
        let mut stmt = self
            .connection
            .prepare("SELECT state FROM users WHERE id = ?1")?;
        let state = stmt
            .query_row(params![user_id], |row| {
                let state_int: i32 = row.get(0)?;
                Ok(UserState::from(state_int))
            })
            .optional()?;
        Ok(state)
    }

    pub fn set_user_state(&self, user_id: i64, new_state: UserState) -> Result<()> {
        let state_int = new_state as i32;
        self.connection.execute(
            "UPDATE users SET state = ?1 WHERE id = ?2",
            params![state_int, user_id],
        )?;
        Ok(())
    }

    pub fn enqueue_user(
        &self,
        user_id: i64,
        search_gender: Gender,
        searcher_gender: Gender,
    ) -> Result<i64> {
        let mut stmt = self.connection.prepare(
            "SELECT user_id FROM queue WHERE searcher_gender = ?1 AND search_gender = ?2 LIMIT 1",
        )?;
        let matching_user_id: Result<i64> = stmt.query_row(
            params![search_gender.clone() as i32, searcher_gender.clone() as i32],
            |row| row.get(0),
        );

        match matching_user_id {
            Ok(match_id) => {
                self.connection
                    .execute("DELETE FROM queue WHERE user_id = ?1", params![match_id])?;
                self.create_chat(user_id, match_id)?;

                return Ok(match_id);
            }
            Err(_) => {
                self.connection.execute(
                    "INSERT INTO queue (user_id, search_gender, searcher_gender) VALUES (?1, ?2, ?3)",
                    params![user_id, search_gender as i32, searcher_gender as i32],
                )?;
            }
        }

        Ok(0)
    }

    pub fn dequeue_user(&self, user_id: i64) -> Result<()> {
        self.connection
            .execute("DELETE FROM queue WHERE user_id = ?1", params![user_id])?;
        Ok(())
    }
}