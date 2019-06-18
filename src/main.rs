#![allow(clippy::too_many_arguments)]
#![feature(decl_macro, proc_macro_hygiene, try_trait)]

extern crate activitypub;
extern crate atom_syndication;
extern crate askama_escape;
extern crate chrono;
extern crate colored;
extern crate ctrlc;
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate gettext_macros;
extern crate gettext_utils;
extern crate guid_create;
extern crate heck;
extern crate lettre;
extern crate lettre_email;
extern crate num_cpus;
extern crate squs_common;
extern crate squs_models;
extern crate reqwest;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_csrf;
extern crate rocket_i18n;
extern crate scheduled_thread_pool;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate serde_qs;
extern crate validator;
#[macro_use]
extern crate validator_derive;
extern crate webfinger;

use diesel::r2d2::ConnectionManager;
use squs_models::{
    db_conn::{DbPool, PragmaForeignKey},
    instance::Instance,
    migrations::IMPORTED_MIGRATIONS,
    Connection, CONFIG,
};
use rocket_csrf::CsrfFairingBuilder;
use scheduled_thread_pool::ScheduledThreadPool;
use std::sync::{Arc, Mutex};

init_i18n!(
    "squs", en, fr
);

mod api;
mod inbox;
mod mail;
#[macro_use]
mod template_utils;
mod routes;
#[macro_use]
extern crate shrinkwraprs;
#[cfg(feature = "test")]
mod test_routes;

include!(concat!(env!("OUT_DIR"), "/templates.rs"));

compile_i18n!();

/// Initializes a database pool.
fn init_pool() -> Option<DbPool> {
    dotenv::dotenv().ok();

    let manager = ConnectionManager::<Connection>::new(CONFIG.database_url.as_str());
    let pool = DbPool::builder()
        .connection_customizer(Box::new(PragmaForeignKey))
        .build(manager)
        .ok()?;
    Instance::cache_local(&pool.get().unwrap());
    Some(pool)
}

fn main() {
    let dbpool = init_pool().expect("main: database pool initialization error");
    if IMPORTED_MIGRATIONS
        .is_pending(&dbpool.get().unwrap())
        .unwrap_or(true)
    {
        panic!(
            r#"
It appear your database migration does not run the migration required
by this version of Plume. To fix this, you can run migrations via
this command:

    plm migration run

Then try to restart Plume.
"#
        )
    }
    let workpool = ScheduledThreadPool::with_name("worker {}", num_cpus::get());

    let mail = mail::init();
    if mail.is_none() && CONFIG.rocket.as_ref().unwrap().environment.is_prod() {
        println!("Warning: the email server is not configured (or not completely).");
        println!("Please refer to the documentation to see how to configure it.");
    }

    let rocket = rocket::custom(CONFIG.rocket.clone().unwrap())
        .mount(
            "/",
            routes![
                routes::comments::create,
                routes::comments::delete,
                routes::comments::activity_pub,
                routes::instance::index,
                routes::instance::admin,
                routes::instance::admin_instances,
                routes::instance::admin_users,
                routes::instance::ban,
                routes::instance::toggle_block,
                routes::instance::update_settings,
                routes::instance::shared_inbox,
                routes::instance::nodeinfo,
                routes::instance::about,
                routes::instance::privacy,
                routes::instance::web_manifest,
                routes::likes::create,
                routes::notifications::notifications,
                routes::posts::activity_details,
                routes::posts::delete,
                routes::reshares::create,
                routes::session::new,
                routes::session::create,
                routes::session::delete,
                routes::session::password_reset_request_form,
                routes::session::password_reset_request,
                routes::session::password_reset_form,
                routes::session::password_reset,
                routes::plume_static_files,
                routes::static_files,
                routes::user::followers,
                routes::user::edit,
                routes::user::update,
                routes::user::delete,
                routes::user::activity_details,
                routes::user::outbox,
                routes::user::inbox,
                routes::user::ap_followers,
                routes::user::new,
                routes::user::create,
                routes::well_known::host_meta,
                routes::well_known::nodeinfo,
                routes::well_known::webfinger,
                routes::errors::csrf_violation
            ],
        )
        .mount(
            "/api/v1",
            routes![
                api::oauth,
                api::apps::create,
                api::comments::for_article,
                api::posts::get,
                api::posts::list,
                api::posts::create,
                api::posts::delete,
                api::posts::fetch_feed,
            ],
        )
        .register(catchers![
            routes::errors::not_found,
            routes::errors::unprocessable_entity,
            routes::errors::server_error
        ])
        .manage(Arc::new(Mutex::new(mail)))
        .manage::<Arc<Mutex<Vec<routes::session::ResetRequest>>>>(Arc::new(Mutex::new(vec![])))
        .manage(dbpool)
        .manage(Arc::new(workpool))
        .manage(include_i18n!())
        .attach(
            CsrfFairingBuilder::new()
                .set_default_target(
                    "/csrf-violation?target=<uri>".to_owned(),
                    rocket::http::Method::Post,
                )
                .add_exceptions(vec![
                    (
                        "/inbox".to_owned(),
                        "/inbox".to_owned(),
                        rocket::http::Method::Post,
                    ),
                    (
                        "/@/<name>/inbox".to_owned(),
                        "/@/<name>/inbox".to_owned(),
                        rocket::http::Method::Post,
                    ),
                    (
                        "/api/<path..>".to_owned(),
                        "/api/<path..>".to_owned(),
                        rocket::http::Method::Post,
                    ),
                ])
                .finalize()
                .expect("main: csrf fairing creation error"),
        );

    #[cfg(feature = "test")]
    let rocket = rocket.mount("/test", routes![test_routes::health,]);
    rocket.launch();
}
