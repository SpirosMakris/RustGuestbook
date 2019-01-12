#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;

use std::collections::HashMap;
use rocket_contrib::templates::Template;

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

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .mount("/", routes![topic_form])
        .attach(Template::fairing())
        .launch();
}