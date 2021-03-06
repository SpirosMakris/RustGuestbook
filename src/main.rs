#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate serde;

use std::collections::HashMap;
use rocket_contrib::templates::Template;
use rocket_contrib::json::{Json, JsonValue};
use chrono::Utc;

use rocket_contrib::databases::rusqlite;
#[database("db_guestbook")]
struct GuestbookDbConn(rusqlite::Connection);

#[derive(FromForm, Serialize)]
struct Post {
    id: Option<i32>,
    reply_id: Option<String>,
    name: String,
    title: String,
    content: String,
}

#[derive(Serialize)]
struct IndexContext {
    title: String,
    announcement: Option<String>,
    posts: Vec<Post>,
}

use rocket::request::Form;
use rocket::response::Redirect;

#[get("/")]
fn index(conn: GuestbookDbConn) -> Template {
  // Make an sql statement and apply a closure to executed result -> iterator
    let mut stmt = conn.prepare("SELECT id, reply_id, name, title, content FROM post").unwrap();
    let post_iter = stmt.query_map(&[],
        |row| {
            Post {
                id: row.get(0),
                reply_id: row.get(1),
                name: row.get(2),
                title: row.get(3),
                content: row.get(4),
           }
       }
    ).unwrap();

    let context = IndexContext {
        title: "Rust Guestbook!".to_string(),
        announcement: Some("Welcome to my guestbook".to_string()),
        posts: post_iter.map(
            |post| post.unwrap()
        ).filter(
            |post| post.reply_id == None
        )
        .collect(),
    };

    Template::render("index", context)
}

#[get("/topic_form")]
fn topic_form() -> Template {
    let mut context = HashMap::new();
    context.insert("title", "Rust GuestBook - topic entry");
    Template::render("post_form", context)
}

#[get("/reply_form/<reply_id>")]
fn reply_form(reply_id: String) -> Template {
    let mut context = HashMap::new();
    context.insert("title", "Post a reply".to_string());
    context.insert("reply_id", reply_id);
    Template::render("post_form", context)
}

#[post("/post", data="<post>")]
fn create_topic(conn: GuestbookDbConn, post: Form<Post>) -> Redirect {
    conn.execute(
        "INSERT INTO post (id, name, title, content, created_time) VALUES (?1, ?2, ?3, ?4, ?5)",
        &[&post.id, &post.name, &post.title, &post.content, &Utc::now().naive_utc().to_string()]
    ).unwrap();

    Redirect::to("/")
}

// Rest API
#[get("/posts", format = "json")]
fn rest_posts(conn: GuestbookDbConn) ->Json<Vec<Post>> {
    let mut stmt = conn.prepare("SELECT * FROM post").unwrap();
    let post_iter = stmt.query_map(
        &[],
        |row| {
            Post {
                id: row.get(0),
                reply_id: row.get(1),
                name: row.get(2),
                title: row.get(3),
                content: row.get(4),
           }
        }
    ).unwrap();

    Json(
        post_iter.map(
            |post| post.unwrap()
        ).collect()
    )
}

#[catch(404)]
fn rest_not_found() -> JsonValue {
    json!(
        {
            "status": "error",
            "reason": "Resource not found."
        }
    )
}

fn main() {
    init_database();

    rocket::ignite()
        .mount("/", routes![index, topic_form, reply_form, create_topic])
        .mount("/api", routes![rest_posts])
        .register(catchers![rest_not_found])
        .attach(Template::fairing())
        .attach(GuestbookDbConn::fairing())
        .launch();
}

fn init_database() {
    let conn = rusqlite::Connection::open("db/guestbook.db3").unwrap();
    conn.execute("DROP TABLE IF EXISTS post", &[]).unwrap();

    conn.execute("CREATE TABLE post (id INTEGER PRIMARY KEY, reply_id, name, title, content, created_time)", &[]).unwrap();

    add_dummy_data(&conn);
}

fn add_dummy_data(conn: &rusqlite::Connection) {
    let post_1 = Post {
        id: None,
        reply_id: Some("reply_id_1".to_string()),
        name: "post_1".to_string(),
        title: "title_1".to_string(),
        content: "content_1 -> lalalalalallal".to_string(),
        //@TODO: Add created time
    };


    let post_2 = Post {
        id: None,
        reply_id: None,
        name: "post_2".to_string(),
        title: "title_2".to_string(),
        content: "content_2 -> More of lalalalalallal".to_string(),
        //@TODO: Add created time
    };

    conn.execute("INSERT INTO post(id, reply_id, name, title, content) VALUES (?1, ?2, ?3, ?4, ?5)",
        &[&post_1.id, &post_1.reply_id, &post_1.name, &post_1.title, &post_1.content]).unwrap();
    
    conn.execute("INSERT INTO post(reply_id, name, title, content) VALUES (?1, ?2, ?3, ?4)",
        &[&post_2.reply_id, &post_2.name, &post_2.title, &post_2.content]).unwrap();
}