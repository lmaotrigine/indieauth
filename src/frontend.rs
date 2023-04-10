use askama::Template;
use rocket::{catch, catchers, fairing::AdHoc, get, http::ContentType, routes, Request};

#[derive(Template)]
#[template(path = "app.html")]
struct App {
    title: String,
    message: String,
}

#[get("/")]
async fn frontend() -> App {
    App {
        title: "Hi".to_string(),
        message: "Loading...".to_string(),
    }
}

#[get("/login")]
async fn login() -> App {
    frontend().await
}

#[get("/sw.js")]
async fn sw() -> (ContentType, &'static str) {
    (ContentType::JavaScript, include_str!("../static/sw.js"))
}

#[derive(Template)]
#[template(path = "notfound.html")]
struct NotFound {
    title: String,
    message: String,
}

#[catch(404)]
async fn not_found(req: &Request<'_>) -> NotFound {
    NotFound {
        title: "Not found".to_string(),
        message: format!("{} not found", req.uri()),
    }
}

pub fn fairing() -> AdHoc {
    async fn f(r: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
        r.register("/", catchers![not_found])
            .mount("/", routes![frontend, login, sw])
    }
    AdHoc::on_ignite("frontend integration", f)
}
