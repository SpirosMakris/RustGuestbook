#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate chrono;
extern crate serde;

use std::collections::HashMap;
use rocket_contrib::templates::Template;
use chrono::Utc;

use rocket_contrib::databases::rusqlite;
#[database("db_guestbook")]
struct GuestbookDbConn(rusqlite::Connection);

#[derive(FromForm, Serialize)]
struct Post {
    name: String,
    title: String,
    content: String,
}

#[derive(Serialize)]
struct TemplateContext {
    title: &'static str,
    index_content: Option<String>,
    posts: Vec<Post>,
}

use rocket::request::Form;
use rocket::response::Redirect;

#[get("/")]
fn index(conn: GuestbookDbConn) -> Template {
    
  // Make an sql statement and apply a closure to executed result -> iterator
    let mut stmt = conn.prepare("SELECT name, title, content FROM post").unwrap();
    let post_iter = stmt.query_map(&[],
       |row| {
           Post {
               name: row.get(0),
               title: row.get(1),
               content: row.get(2),
           }
       } 
    ).unwrap();

    let context = TemplateContext {
        title: "Rust Guestbook!",
        index_content: Some("Welcome to my guestbook".to_string()),
        posts: post_iter.map(
            |post| post.unwrap()
        ).collect(),
    };

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