extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate tokio_postgres;

use actix::prelude::*;
use actix_web::{http, server, App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse};
use futures::Future;

mod db;
use self::db::{AddUser, PgConnection, ReadUsers};

struct State {
  db: Addr<PgConnection>,
}

fn index(req: &HttpRequest<State>) -> &'static str {
  "Hello world"
}

fn read_from_db(req: &HttpRequest<State>) -> FutureResponse<HttpResponse> {
  let mut resp = HttpResponse::build_from(req);
  req
    .state()
    .db
    .send(ReadUsers {
      search_name: String::from("some_name"),
    }).from_err()
    .and_then(move |res| match res {
      Ok(data) => {
        println!("{:?}", data);

        Ok(
          resp
            .header(http::header::SERVER, "Actix")
            .content_type("application/json")
            .body("Everything works"),
        )
      }
      Err(_) => Ok(HttpResponse::InternalServerError().into()),
    }).responder()
}

fn write_to_db(req: &HttpRequest<State>) -> FutureResponse<HttpResponse> {
  let mut resp = HttpResponse::build_from(req);

  req
    .state()
    .db
    .send(AddUser {
      user_name: req.match_info().get("name").unwrap().to_string(),
    }).from_err()
    .and_then(move |res| match res {
      Ok(_) => Ok(
        resp
          .header(http::header::SERVER, "Actix")
          .content_type("application/json")
          .body("Everything works"),
      ),
      Err(_) => Ok(HttpResponse::InternalServerError().into()),
    }).responder()
}

fn main() {
  env_logger::init();

  let sys = actix::System::new("server");
  let db_url = "";

  server::new(move || {
    App::with_state(State {
      db: PgConnection::connect(db_url),
    }).resource("/", |r| {
      // continue
      r.route().f(index)
    }).resource("/write/{name}", |r| {
      // write to db
      r.route().f(write_to_db)
    }).resource("/read", |r| {
      //
      r.route().f(read_from_db)
    })
  }).bind("0.0.0.0:3000")
  .unwrap()
  .start();

  println!("Started http server: 127.0.0.1:3000");
  let _ = sys.run();
}
