use activitypub::object::Note;
use rocket::{
    request::LenientForm,
    response::{Flash, Redirect},
};
use template_utils::Ructe;
use validator::Validate;

use std::time::Duration;

use squs_common::{
    activity_pub::{broadcast, ActivityStream, ApRequest},
    utils,
};
use squs_models::{
    comments::*, inbox::inbox, instance::Instance, mentions::Mention,
    posts::Post, safe_string::SafeString, users::User, Error, PlumeRocket,
};
use routes::errors::ErrorPage;
use template_utils::IntoContext;

#[derive(Default, FromForm, Debug, Validate)]
pub struct NewCommentForm {
    pub responding_to: Option<i32>,
    #[validate(length(min = "1", message = "Your reply can't be empty"))]
    pub content: String,
    pub warning: String,
}

#[post("/c/<id>/reply", data = "<form>")]
pub fn create(
    id: i32,
    form: LenientForm<NewCommentForm>,
    user: User,
    rockets: PlumeRocket,
) -> Result<Flash<Redirect>, Ructe> {
    let conn = &*rockets.conn;
    let post = Post::get(&*conn, id).expect("comments::create: post error");
    form.validate()
        .map(|_| {
            let (html, mentions, _hashtags) = utils::md_to_html(
                form.content.as_ref(),
                Some(
                    &Instance::get_local()
                        .expect("comments::create: local instance error")
                        .public_domain,
                ),
                true,
                None,
            );
            let comm = Comment::insert(
                &*conn,
                NewComment {
                    content: SafeString::new(html.as_ref()),
                    in_response_to_id: form.responding_to,
                    post_id: post.id,
                    author_id: user.id,
                    ap_id: String::new(),
                    sensitive: !form.warning.is_empty(),
                    spoiler_text: form.warning.clone(),
                    public_visibility: true,
                },
            )
            .expect("comments::create: insert error");
            let new_comment = comm
                .create_activity(&rockets)
                .expect("comments::create: activity error");

            // save mentions
            for ment in mentions {
                Mention::from_activity(
                    &*conn,
                    &Mention::build_activity(&rockets, &ment)
                        .expect("comments::create: build mention error"),
                    comm.id,
                    false,
                    true,
                )
                .expect("comments::create: mention save error");
            }

            comm.notify(&*conn).expect("comments::create: notify error");

            // federate
            let dest = User::one_by_instance(&*conn).expect("comments::create: dest error");
            let user_clone = user.clone();
            rockets
                .worker
                .execute(move || broadcast(&user_clone, new_comment, dest));

            Flash::success(
                Redirect::to(uri!(super::instance::index)),
                i18n!(&rockets.intl.catalog, "Your comment has been posted."),
            )
        })
        .map_err(|_| {
            // TODO: de-duplicate this code
            // TODO: show the error
            let comments = CommentTree::from_post(&*conn, &post, Some(&user))
                .expect("comments::create: comments error");

            render!(comments::details(
                &rockets.to_context(),
                comments
            ))
        })
}

#[post("/c/<id>/delete")]
pub fn delete(
    id: i32,
    user: User,
    rockets: PlumeRocket,
) -> Result<Flash<Redirect>, ErrorPage> {
    if let Ok(comment) = Comment::get(&*rockets.conn, id) {
        if comment.author_id == user.id {
            let dest = User::one_by_instance(&*rockets.conn)?;
            let delete_activity = comment.build_delete(&*rockets.conn)?;
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
                    user.rotate_keypair(&conn)
                        .expect("Failed to rotate keypair");
                });
        }
    }
    Ok(Flash::success(
        Redirect::to(uri!(super::instance::index)),
        i18n!(&rockets.intl.catalog, "Your comment has been deleted."),
    ))
}

#[get("/c/<id>")]
pub fn activity_pub(
    id: i32,
    _ap: ApRequest,
    rockets: PlumeRocket,
) -> Option<ActivityStream<Note>> {
    Comment::get(&*rockets.conn, id)
        .and_then(|c| c.to_activity(&rockets))
        .ok()
        .map(ActivityStream::new)
}
