use rocket::{
    fairing::{AdHoc, Fairing, Info, Kind},
    http::Header,
};
use rusty_ulid::generate_ulid_string;

#[derive(Default)]
pub struct RequestId;

#[rocket::async_trait]
impl Fairing for RequestId {
    fn info(&self) -> rocket::fairing::Info {
        Info {
            name: "Request ID",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut rocket::Request<'_>, _: &mut rocket::Data<'_>) {
        match request.headers().get_one("X-Request-Id") {
            Some(_) => {}
            None => {
                let reqid = generate_ulid_string();
                request.add_header(Header::new("X-Request-Id", reqid));
            }
        };
    }

    async fn on_response<'r>(
        &self,
        request: &'r rocket::Request<'_>,
        response: &mut rocket::Response<'r>,
    ) {
        if let Some(reqid) = request.headers().get_one("X-Request-Id") {
            response.set_header(Header::new("X-Request-Id", reqid));
        }
    }
}

pub fn static_files() -> AdHoc {
    async fn f(rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build> {
        let asset_path: String = rocket.figment().extract_inner("asset_path").unwrap();
        rocket.mount("/static", rocket::fs::FileServer::from(asset_path))
    }
    AdHoc::on_ignite("static fileserver", f)
}
