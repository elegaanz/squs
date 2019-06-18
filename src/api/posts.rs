use rocket_contrib::json::Json;

use crate::api::{authorization::*, Api};
use squs_common::activity_pub::broadcast;
use squs_models::{
    db_conn::DbConn, instance::Instance,
    posts::*, safe_string::SafeString, users::User, PlumeRocket,
};
use serde::{Serialize, Deserialize};
use std::{thread, str::FromStr};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct NewPostData {
    title: String,
    url: String,
    subtitle: Option<String>,
    content: String,
    license: Option<String>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PostData {
    id: i32,
    title: String,
    url: String,
    subtitle: String,
    content: String,
    creation_date: String,
    license: String,
}

#[get("/posts/<id>")]
pub fn get(id: i32, conn: DbConn) -> Api<PostData> {
    let post = Post::get(&conn, id)?;

    Ok(Json(PostData {
        creation_date: post.creation_date.format("%Y-%m-%d").to_string(),
        id: post.id,
        title: post.title,
        subtitle: post.subtitle,
        content: post.content.to_string(),
        license: post.license,
        url: post.url,
    }))
}

#[get("/posts?<title>&<subtitle>&<content>")]
pub fn list(title: Option<String>, subtitle: Option<String>, content: Option<String>, conn: DbConn) -> Api<Vec<PostData>> {
    Ok(Json(
        Post::list_filtered(&conn, title, subtitle, content)?
            .into_iter()
            .map(|p| {
                PostData {
                    creation_date: p.creation_date.format("%Y-%m-%d").to_string(),
                    id: p.id,
                    title: p.title,
                    subtitle: p.subtitle,
                    content: p.content.to_string(),
                    license: p.license,
                    url: p.url,
                }
            })
            .collect(),
    ))
}

#[post("/posts", data = "<payload>")]
pub fn create(
    auth: Authorization<Write, Post>,
    payload: Json<NewPostData>,
    rockets: PlumeRocket,
) -> Api<PostData> {
    let conn = &*rockets.conn;
    let worker = &rockets.worker;

    let author = User::get(conn, auth.0.user_id)?;

    let post = Post::insert(
        conn,
        NewPost {
            author_id: author.id,
            url: payload.url.clone(),
            title: payload.title.clone(),
            content: SafeString::new(&payload.content),
            license: payload.license.clone().unwrap_or_else(|| {
                Instance::get_local()
                    .map(|i| i.default_license)
                    .unwrap_or_else(|_| String::from("CC-BY-SA"))
            }),
            ap_id: String::new(),
            subtitle: payload.subtitle.clone().unwrap_or_default(),
        },
    )?;

    let act = post.create_activity(&*conn)?;
    let dest = User::one_by_instance(&*conn)?;
    worker.execute(move || broadcast(&author, act, dest));

    Ok(Json(PostData {
        creation_date: post.creation_date.format("%Y-%m-%d").to_string(),
        id: post.id,
        title: post.title,
        subtitle: post.subtitle,
        content: post.content.to_string(),
        license: post.license,
        url: post.url,
    }))
}

#[delete("/posts/<id>")]
pub fn delete(auth: Authorization<Write, Post>, rockets: PlumeRocket, id: i32) -> Api<()> {
    let author = User::get(&*rockets.conn, auth.0.user_id)?;
    if let Ok(post) = Post::get(&*rockets.conn, id) {
        if post.author_id == author.id {
            post.delete(&*rockets.conn)?;
        }
    }
    Ok(Json(()))
}

#[get("/posts/feed?<url>")]
pub fn fetch_feed(url: String, conn: DbConn, auth: Authorization<Write, Post>) -> Api<()> {
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(1000 * 60 * 2)); // wait two minutes to be sure the feed is up to date
        let feed = &reqwest::get(&url).expect("can't fetch feed").text().expect("read text");
        let feed = atom_syndication::Feed::from_str(&feed).expect("invalid feed");
        let user = User::get(&*conn, auth.0.user_id).expect("can't find author");
        for entry in feed.entries() {
            let post = Post::insert(&*conn, NewPost {
                url: entry.links().into_iter().find(|l| l.mime_type() == Some("text/html".into())).or_else(|| entry.links().into_iter().next()).expect("no url").href().into(),
                author_id: user.id,
                title: entry.title().into(),
                content: SafeString::new(entry.content().and_then(|c| c.value()).unwrap_or_default()),
                license: entry.rights().unwrap_or_default().into(),
                ap_id: String::new(),
                subtitle: entry.summary().unwrap_or_default().into(),
            });
            match post {
                Ok(post) => {
                    let act = post.create_activity(&*conn).expect("couldn't generate Create Article");
                    let dest = User::one_by_instance(&*conn).expect("can't list dest");
                    broadcast(&user, act, dest);
                },
                Err(e) => println!("post was not ok {:?}", e)
            }
        }
    });
    Ok(Json(()))
}
