use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub struct Connection {
    pub file: String,
    pub state: Option<SqliteConnection>,
}

impl Connection {
    pub fn new(file: Option<&str>) -> Self {
        let mut connection = match file {
            Some(name) => Self {
                file: name.to_string(),
                state: None,
            },
            None => Self::default(),
        };

        match connection.connect() {
            Ok(_) => connection,
            Err(error) => {
                println!("Connection Error: {}", error);
                connection
            }
        }
    }

    fn connect(&mut self) -> Result<(), diesel::ConnectionError> {
        use diesel::prelude::*;

        let file = &self.file;
        let connection = SqliteConnection::establish(file)?;

        self.state = Some(connection);
        Ok(())
    }
}

impl Default for Connection {
    fn default() -> Self {
        dotenv().ok();

        let file = env::var("DATABASE_FILE").expect("DATABASE_FILE must be set");

        Self { file, state: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_url_parameter() {
        let url = Some("cement_test.sqlite3");
        let result = Connection::new(url);

        assert_eq!("cement_test.sqlite3", result.file);
    }

    #[test]
    fn without_url_parameter() {
        let result = Connection::new(None);
        assert_eq!("cement_development.sqlite3", result.file);
    }

    #[test]
    fn doesnt_panic_with_bad_url() {
        let url = Some("uh_oh\0");
        let _ = Connection::new(url);
    }
}
