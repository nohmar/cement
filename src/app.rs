use std::io::ErrorKind;

use clap::Parser;
use diesel::{prelude::*, result::Error, sqlite::SqliteConnection};

use crate::{
    cli::Cli,
    models::{Idiom, NewIdiom},
};

#[derive(Debug)]
pub struct App<'a> {
    pub cli: Cli,
    args: Option<Vec<&'a str>>,
}

impl<'a> App<'a> {
    pub fn new(args: Option<Vec<&'a str>>) -> App<'a> {
        let mut app = App {
            cli: Cli {
                list: false,
                destroy: None,
                example: None,
                phrase: None,
            },
            args,
        };

        let cli = app.parse_args();
        app.cli = cli;

        app
    }

    fn parse_args(&self) -> Cli {
        match &self.args {
            Some(arg_vec) => Cli::parse_from(arg_vec),
            None => Cli::parse(),
        }
    }

    pub fn store(&self, connection: &mut SqliteConnection) -> Result<usize, Error> {
        use crate::schema::idioms;

        let phrase = self
            .cli
            .phrase
            .as_ref()
            .ok_or("Phrase is required.")
            .unwrap()
            .to_string();

        let example = self.cli.example.as_ref().map(|example| example.to_string());
        let new_idiom = NewIdiom { phrase, example };

        diesel::insert_into(idioms::table)
            .values(&new_idiom)
            .execute(connection)
    }

    pub fn destroy(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::schema::idioms::dsl::*;
        use diesel::prelude::*;

        let idiom_to_delete = self.cli.destroy.as_ref().unwrap();

        let existing_idiom = idioms
            .filter(phrase.eq(idiom_to_delete))
            .select(Idiom::as_select())
            .first(connection);

        match existing_idiom {
            Ok(result) => {
                diesel::delete(idioms.filter(phrase.eq(idiom_to_delete))).execute(connection)?;

                println!(
                    "Deleted {}, {}",
                    result.phrase,
                    result.example.unwrap_or("no example found.".to_string())
                );

                Ok(())
            }
            Err(_) => {
                println!("{} doesn't exist.", idiom_to_delete);
                Ok(())
            }
        }
    }

    pub fn output_from_args(
        &self,
        connection: &mut SqliteConnection,
    ) -> Result<String, std::io::Error> {
        use crate::schema::idioms::dsl::*;

        match self.cli.list {
            true => {
                let select = idioms
                    .select(Idiom::as_select())
                    .load(connection)
                    .optional()
                    .unwrap();

                let json = serde_json::to_string_pretty(&select).unwrap();

                Ok(json)
            }
            false => match self.store(connection) {
                Ok(_) => {
                    let result = "Stored phrase ".to_owned() + self.cli.phrase.as_ref().unwrap();
                    Ok(result.to_string())
                }
                Err(error) => {
                    let message = "Something went wrong: ".to_owned() + &error.to_string();

                    Err(std::io::Error::new(ErrorKind::Other, message))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::Connection;
    use crate::models::Idiom;
    use dotenvy::dotenv;
    use std::env;
    use std::time::SystemTime;

    fn get_conn() -> Result<SqliteConnection, Box<dyn std::error::Error>> {
        dotenv().ok();

        let database_url = env::var("TEST_DATABASE_FILE").expect("TEST_DATABASE_FILE must be set");

        Connection::new(Some(&database_url))
            .state
            .ok_or(Box::new(std::io::Error::new(ErrorKind::Other, "Failed.")))
    }

    #[test]
    #[should_panic]
    fn store_errors_without_phrase() {
        let app = App::new(None);

        let conn = &mut get_conn().unwrap();

        let _ = app.store(conn);
    }

    #[test]
    fn persists_phrase_with_example() {
        use crate::schema::idioms::dsl::*;
        use diesel::prelude::*;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Unable to get time since Unix epoch.")
            .as_secs()
            .to_string();

        let new_phrase = "hello-".to_string() + &now;

        let app = App::new(Some(vec![
            "",
            &new_phrase,
            "-e",
            "An example of an example.",
        ]));

        let conn = &mut get_conn().unwrap();

        let result = app.store(conn);

        let query_results = idioms
            .filter(phrase.eq(new_phrase))
            .select(Idiom::as_select())
            .first(conn)
            .expect("Error loading idiom results.");

        assert_eq!("hello-".to_string() + &now, query_results.phrase);
        assert_eq!("An example of an example.", query_results.example.unwrap());
    }

    #[test]
    fn output_list_of_idioms() -> Result<(), Box<dyn std::error::Error>> {
        let args = Some(vec!["", "-l"]);
        let app = App::new(args);
        let conn = &mut get_conn().unwrap();

        app.output_from_args(conn)?;

        Ok(())
    }

    #[test]
    fn destroy_an_existing_idiom() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Unable to get time since Unix epoch.")
            .as_secs()
            .to_string();

        let new_phrase = "new-phrase-".to_string() + &now;

        let mut app = App::new(Some(vec![
            "",
            &new_phrase,
            "-e",
            "An example of an example.",
        ]));

        let conn = &mut get_conn().unwrap();

        let _ = app.store(conn);

        app.cli = Cli::parse_from(vec!["", "-d", &new_phrase]);
        let _ = app.destroy(conn);
    }

    #[test]
    fn destroy_fails_without_existence() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Unable to get time since Unix epoch.")
            .as_secs()
            .to_string();

        let new_phrase = "new-phrase-".to_string() + &now;

        let mut app = App::new(Some(vec![
            "",
            &new_phrase,
            "-e",
            "An example of an example.",
        ]));

        let conn = &mut get_conn().unwrap();

        app.cli = Cli::parse_from(vec!["", "-d", &new_phrase]);
        let _ = app.destroy(conn);
    }
}
