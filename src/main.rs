use rocket::tokio::time::{sleep, Duration};
use log::{info};
use std::path::{Path, PathBuf};
use rocket::fs::NamedFile;
use serde::{Deserialize, Serialize};
use serde_json::{json};
use rocket::http::Status;
use rocket::request::{Outcome, self, Request, FromRequest};

use rocket::serde::json::Json;
use rocket::fs::TempFile;

#[macro_use] extern crate rocket;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u64,
    name: String,
    age: u8,
    phones: Vec<String>,
}

struct ApiKey(String);

#[derive(Debug)]
enum ApiKeyError {
    BadCount,
    Missing,
    Invalid,
}

fn is_valid(key: &str) -> bool {
    key.eq("52e50a2c-e4e0-4ffc-9c3a-5cb2c2a70e6f")
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ApiKeyError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<_> = request.headers().get("x-api-key").collect();
        match keys.len() {
            0 => Outcome::Failure((Status::BadRequest, ApiKeyError::Missing)),
            1 if is_valid(keys[0]) => Outcome::Success(ApiKey(keys[0].to_string())),
            1 => Outcome::Failure((Status::BadRequest, ApiKeyError::Invalid)),
            _ => Outcome::Failure((Status::BadRequest, ApiKeyError::BadCount)),
        }
    }
}

#[get("/sensitive")]
fn sensitive(key: ApiKey) -> &'static str {
    "Sensitive data."
}

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Waited for seconds {}", seconds)
}

#[get("/hello/<name>/delay/<seconds>")]
async fn hello(name: &str, seconds: u64) -> String {
    info!("{}, wait for {} seconds...", name, seconds);
    sleep(Duration::from_secs(seconds)).await;
    format!("the name is {}", name)
}

// multiple segments
#[get("/page/<file..>")]
async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("/tmp/static/").join(file)).await.ok()
}

//ignore segments...
#[get("/foo/<_>/bar")]
fn foo_bar() -> &'static str {
    "Foo ____ bar!"
}

#[get("/some/<_..>")]
fn everything() -> &'static str {
    "Everything mapped!"
}

// forwarding...
#[get("/user/<id>")]
fn user(id: usize) -> String {
    format!("usize = {}", id)
}
#[get("/user/<id>", rank = 2)]
fn user_int(id: isize) -> String {
    format!("isize = {}", id)
}
#[get("/user/<id>", rank = 3)]
fn user_str(id: &str) -> String {
    let user = User {
        id: 0,
        name: "Itanor Strapazzon".to_owned(),
        age: 39,
        phones: vec![
            "+55 4899676 6015".to_string(),
        ]
    };
    match serde_json::to_string(&user) {
        Ok(json) => json,
        Err(_err) => json!({
            "error": "cannot convert user to json"
        }).to_string()
    }
}

#[post("/user", format = "json", data = "<user>")]
fn new_user(user: Json<User>) {
   println!("{:?}", user); 
}

#[post("/upload", data = "<file>")]
async fn upload(mut file: TempFile<'_>) -> std::io::Result<()> {
    file.persist_to("/tmp/upload/file.txt").await;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            delay, hello, files, foo_bar, 
            everything, user, user_int, user_str,
            sensitive, new_user, upload
        ])
}

