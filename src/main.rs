pub mod models;
pub mod schema;

mod app;
mod cli;
mod connection;

use app::App;
use connection::Connection;
use diesel::r2d2::ConnectionManager;
use diesel::sqlite::SqliteConnection;

fn main() {
    let connection = Connection::new(None);
    let manager = ConnectionManager::<SqliteConnection>::new(connection.file);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool.");

    let pool = pool.clone();
    let app = App::new(None);

    let conn = &mut pool.get().unwrap();

    println!("{}", app.output_from_args(conn).unwrap());
}
