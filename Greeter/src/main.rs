#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::request::Form;
use rocket::response::content::HTML;

const FORM_TEMPLATE: &'static str = "
<html>
<head>
<title> Rocket Test </title>
</head>
<body>
<form action='name' method='POST'>
What is your name?<br />
<input type='text' name='name'>
</form>
</body>";

#[get("/")]
fn index() -> HTML<String> {
    HTML(String::from(FORM_TEMPLATE))
}

#[derive(FromForm)]
struct FormName {
    name: String
}

#[post("/name", data="<name>")]
fn name(name: Form<FormName>) -> String {
    let name_data = name.get();
    format!("Hello, {}", name_data.name)
}

fn main() {
    rocket::ignite().mount("/", routes![index, name]).launch();
}