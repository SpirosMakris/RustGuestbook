#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate chrono;

use std::collections::HashMap;
use rocket_contrib::templates::Template;
use chrono::Utc;

use rocket_contrib::databases::rusqlite;
#[database("db_guestbook")]
struct GuestbookDbConn(rusqlite::Connection);

#[derive(FromForm)]
struct Post {
    name: String,
    title: String,
    content: String,
}

use rocket::request::Form;
use rocket::response::Redirect;

#[get("/")]
fn index() -> Template {
    let mut context = HashMap::new();
    context.insert("title", "Rust GuestBook");
    context.insert("body", "Welcome to my guestbook.");
    Template::render("index", context)
}

#[get("/topic_form")]
fn topic_form() -> Template {
    let mut context = HashMap::new();
    context.insert("title", "Rust GuestBook - topic entry");
    Template::render("topic_form", context)
}

#[post("/topic", data="<post>")]
fn create_topic(conn: GuestbookDbConn, post: Form<Post>) -> Redirect {
    // let post_data = post.get();
    conn.execute(
        "INSERT INTO post (name, title, content, created_time) VALUES (?1, ?2, ?3, ?4)",
        &[&post.name, &post.title, &post.content, &Utc::now().naive_utc().to_string()]
    ).unwrap();

    Redirect::to("/")
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/", routes![topic_form])
        .mount("/", routes![create_topic])
        .attach(Template::fairing())
        .attach(GuestbookDbConn::fairing())
        .launch();
}