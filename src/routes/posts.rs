use rocket::response::{Flash, Redirect};
use rocket_i18n::I18n;
use std::time::Duration;

use squs_common::activity_pub::{broadcast, ActivityStream, ApRequest};
use squs_models::{
    inbox::inbox,
    posts::*,
    users::User,
    Error, PlumeRocket,
};
use routes::errors::ErrorPage;

#[get("/p/<id>", rank = 3)]
pub fn activity_details(
    id: i32,
    _ap: ApRequest,
    rockets: PlumeRocket,
) -> Result<ActivityStream<LicensedArticle>, Option<String>> {
    let conn = &*rockets.conn;
    let post = Post::get(&*conn, id).map_err(|_| None)?;
    Ok(ActivityStream::new(
        post.to_activity(&*conn)
            .map_err(|_| String::from("Post serialization error"))?,
    ))
}

#[post("/p/<id>/delete")]
pub fn delete(
    id: i32,
    rockets: PlumeRocket,
    intl: I18n,
) -> Result<Flash<Redirect>, ErrorPage> {
    let user = rockets.user.clone().unwrap();
    let post = Post::get(&rockets.conn, id);

    if let Ok(post) = post {
        if !post.author_id == user.id {
            return Ok(Flash::error(
                Redirect::to(uri!(super::instance::index)),
                i18n!(intl.catalog, "You are not allowed to delete this article."),
            ));
        }

        let dest = User::one_by_instance(&*rockets.conn)?;
        let delete_activity = post.build_delete(&*rockets.conn)?;
        inbox(
            &rockets,
            serde_json::to_value(&delete_activity).map_err(Error::from)?,
        )?;

        let user_c = user.clone();
        rockets
            .worker
            .execute(move || broadcast(&user_c, delete_activity, dest));
        let conn = rockets.conn;
        rockets
            .worker
            .execute_after(Duration::from_secs(10 * 60), move || {
                user.rotate_keypair(&*conn)
                    .expect("Failed to rotate keypair");
            });

        Ok(Flash::success(
            Redirect::to(uri!(super::instance::index)),
            i18n!(intl.catalog, "Your article has been deleted."),
        ))
    } else {
        Ok(Flash::error(Redirect::to(
            uri!(super::instance::index),
        ), i18n!(intl.catalog, "It looks like the article you tried to delete doesn't exist. Maybe it is already gone?")))
    }
}
