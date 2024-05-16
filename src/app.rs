use std::io::ErrorKind;

use clap::Parser;
use diesel::{prelude::*, result::Error, sqlite::SqliteConnection};

use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;

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

    pub fn store(
        &self,
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<usize, Error> {
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
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
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
        connection: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
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
            false => match self.cli.destroy.as_ref() {
                Some(_) => {
                    let _ = self.destroy(connection);
                    Ok("".to_string())
                }
                None => {
                    return match self.store(connection) {
                        Ok(_) => {
                            let result =
                                "Stored phrase ".to_owned() + self.cli.phrase.as_ref().unwrap();
                            Ok(result.to_string())
                        }
                        Err(error) => {
                            let message = "Something went wrong: ".to_owned() + &error.to_string();

                            Err(std::io::Error::new(ErrorKind::Other, message))
                        }
                    };
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Idiom;
    use crate::Connection;
    use dotenvy::dotenv;
    use r2d2::Pool;
    use std::env;
    use std::sync::OnceLock;
    use std::time::SystemTime;

    fn get_pool() -> &'static Pool<ConnectionManager<SqliteConnection>> {
        dotenv().ok();

        static POOL: OnceLock<Pool<ConnectionManager<SqliteConnection>>> = OnceLock::new();

        POOL.get_or_init(|| {
            let database_url =
                env::var("TEST_DATABASE_FILE").expect("TEST_DATABASE_FILE must be set");
            let connection = Connection::new(Some(&database_url));
            let manager = ConnectionManager::<SqliteConnection>::new(connection.file);

            let pool = r2d2::Pool::builder()
                .max_size(1)
                .build(manager)
                .expect("Failed to create DB pool.");

            pool
        })
    }

    #[test]
    #[should_panic]
    fn store_errors_without_phrase() {
        let app = App::new(None);

        let pool = get_pool();

        let conn = &mut pool.get().unwrap();
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

        let pool = get_pool();
        let conn = &mut pool.get().unwrap();

        let new_phrase = "hello-".to_string() + &now;
        let app_args = vec!["", &new_phrase, "-e", "An example of an example."];

        let app = App::new(Some(app_args));

        let result = app.store(conn);

        // FIXME: Spurious failures occur on this test with:
        // Err(DatabaseError(Unknown, "database is locked"))
        println!("{:?}", result);

        let query_results = idioms
            .filter(phrase.eq(new_phrase))
            .select(Idiom::as_select())
            .first(conn)
            .expect("Error loading idiom results.");

        assert_eq!("hello-".to_string() + &now, query_results.phrase);
        assert_eq!("An example of an example.", query_results.example.unwrap());
    }

    #[test]
    fn output_list_of_idioms() {
        let args = Some(vec!["", "-l"]);
        let app = App::new(args);

        let pool = get_pool();
        let conn = &mut pool.get().unwrap();

        let _ = app.output_from_args(conn);
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

        let pool = get_pool();
        let conn = &mut pool.get().unwrap();

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

        let pool = get_pool();
        let conn = &mut pool.get().unwrap();

        app.cli = Cli::parse_from(vec!["", "-d", &new_phrase]);
        let _ = app.destroy(conn);
    }

    #[test]
    fn destory_flag() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Unable to get time since Unix epoch.")
            .as_secs()
            .to_string();

        let new_phrase = "new-phrase-".to_string() + &now;

        let args = Some(vec!["", "-d", &new_phrase]);
        let app = App::new(args);
        let pool = get_pool();
        let conn = &mut pool.get().unwrap();

        let _ = app.output_from_args(conn);
    }
}
