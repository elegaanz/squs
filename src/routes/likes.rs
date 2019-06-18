use rocket::response::Redirect;

use squs_common::activity_pub::broadcast;
use squs_models::{
    comments::Comment, inbox::inbox, likes, users::User, Error, PlumeRocket,
};
use routes::errors::ErrorPage;

#[post("/c/<id>/like")]
pub fn create(
    id: i32,
    user: User,
    rockets: PlumeRocket,
) -> Result<Redirect, ErrorPage> {
    let conn = &*rockets.conn;
    let comm = Comment::get(&*conn, id)?;

    if !user.has_liked(&*conn, &comm)? {
        let like = likes::Like::insert(&*conn, likes::NewLike::new(&comm, &user))?;
        like.notify(&*conn)?;

        let dest = User::one_by_instance(&*conn)?;
        let act = like.to_activity(&*conn)?;
        rockets.worker.execute(move || broadcast(&user, act, dest));
    } else {
        let like = likes::Like::find_by_user_on_comment(&*conn, user.id, comm.id)?;
        let delete_act = like.build_undo(&*conn)?;
        inbox(
            &rockets,
            serde_json::to_value(&delete_act).map_err(Error::from)?,
        )?;

        let dest = User::one_by_instance(&*conn)?;
        rockets
            .worker
            .execute(move || broadcast(&user, delete_act, dest));
    }

    Ok(Redirect::to(
        uri!(super::instance::index),
    ))
}
