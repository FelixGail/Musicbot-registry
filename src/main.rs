#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::sync::RwLock;
use std::time::Duration;

use rocket::{http::Status, State};
use rocket_contrib::json::Json;

use rocket_cors::Error;

mod lib;
use crate::lib::{AddressEntry, BotInstance, Registry, RemoteAddress};

static TTL_SECS: u64 = 300;
static CAPACITY: usize = 10000;

type LockRegistry = RwLock<Registry>;

#[get("/")]
fn index(registry: State<LockRegistry>, addr: RemoteAddress) -> Json<Vec<AddressEntry>> {
    let reg = registry.read().expect("Could not retrieve read lock");
    if let Some((vec, dirty)) = reg.get(&addr.ip()) {
        std::mem::drop(reg);
        if dirty {
            let mut write_reg = registry.write().expect("Could not retrieve read lock");
            write_reg.clean_key(&addr.ip());
        }
        return Json(vec);
    }
    Json(Vec::with_capacity(0))
}

#[post("/", data = "<instance>")]
fn post(registry: State<LockRegistry>, addr: RemoteAddress, instance: Json<BotInstance>) -> Status {
    let mut reg = registry.write().expect("could not lock registry");
    if reg.insert(addr.ip(), instance.into_inner()) {
        return Status::Accepted;
    }
    Status::InternalServerError
}

pub fn main() -> Result<(), Error> {
    let cors = rocket_cors::CorsOptions {
        max_age: Some(86400),
        ..Default::default()
    }
    .to_cors()?;

    rocket::ignite()
        .manage(RwLock::new(Registry::create(
            CAPACITY,
            Duration::from_secs(TTL_SECS),
        )))
        .mount("/", routes![index, post])
        .attach(cors)
        .launch();
    Ok(())
}
