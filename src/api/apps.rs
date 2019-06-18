use rocket_contrib::json::Json;

use crate::api::Api;
use squs_common::utils::random_hex;
use squs_models::{apps::*, db_conn::DbConn};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct NewAppData {
    pub name: String,
    pub website: Option<String>,
    pub redirect_uri: Option<String>,
}

#[post("/apps", data = "<data>")]
pub fn create(conn: DbConn, data: Json<NewAppData>) -> Api<App> {
    let client_id = random_hex();
    let client_secret = random_hex();
    let app = App::insert(
        &*conn,
        NewApp {
            name: data.name.clone(),
            client_id,
            client_secret,
            redirect_uri: data.redirect_uri.clone(),
            website: data.website.clone(),
        },
    )?;

    Ok(Json(app))
}
