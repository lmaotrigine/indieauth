use rocket::get;

#[get("/.well-known/botinfo")]
pub async fn botinfo() -> &'static str {
    include_str!("./botinfo.txt")
}

#[get("/robots.txt")]
pub async fn robots() -> String {
    include_str!("./robots.txt").to_string()
}

#[get("/security.txt")]
pub async fn security() -> String {
    include_str!("./security.txt").to_string()
}
