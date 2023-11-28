pub mod models;
pub mod schema;

mod app;
mod cli;
mod connection;

use app::App;
use connection::Connection;

fn main() {
    let app = App::new(None);

    let connection = Connection::new(None);
    let connection_state = &mut connection.state.unwrap();

    println!("{}", app.output_from_args(connection_state).unwrap());
}
