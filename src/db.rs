use std::io;

use actix::fut;
use actix::prelude::*;
use futures::{Future, Stream};
use tokio_postgres::{connect, Client, Statement, TlsMode};

pub struct PgConnection {
  pg_client: Option<Client>,
  // write all the statements
  write_to_db: Option<Statement>,
}

impl Actor for PgConnection {
  type Context = Context<Self>;
}

impl PgConnection {
  pub fn connect(db_url: &str) -> Addr<PgConnection> {
    let pg_connection = connect(db_url.parse().unwrap(), TlsMode::None);

    PgConnection::create(move |ctx| {
      let pg_actor = PgConnection {
        pg_client: None,
        write_to_db: None,
      };

      pg_connection
        .map_err(|_| panic!("Can not connect to postgresql"))
        .into_actor(&pg_actor)
        .and_then(|(pg_client, conn), pg_actor, ctx| {
          // implement all statements which we may need

          ctx.wait(
            pg_client
              .prepare("INSERT INTO person (name, data) VALUES ($1, '')")
              .map_err(|_| ())
              .into_actor(pg_actor)
              .and_then(|st, pg_actor, _| {
                pg_actor.write_to_db = Some(st);
                fut::ok(())
              }),
          );

          // end for prepared statements

          pg_actor.pg_client = Some(pg_client);
          Arbiter::spawn(conn.map_err(|e| panic!("{}", e)));
          fut::ok(())
        }).wait(ctx);

      pg_actor
    })
  }
}

// All statements to run from the client
pub struct AddUser {
  pub user_name: String,
}

impl Message for AddUser {
  type Result = io::Result<()>;
}

impl Handler<AddUser> for PgConnection {
  type Result = ResponseFuture<(), io::Error>;

  fn handle(&mut self, data: AddUser, _: &mut Self::Context) -> Self::Result {
    Box::new(
      self
        .pg_client
        .as_mut()
        .unwrap()
        .query(
          self.write_to_db.as_ref().unwrap(),
          &[&data.user_name.as_str()],
        ).into_future()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.0))
        .and_then(|(_, _)| Ok(())),
    )
  }
}
