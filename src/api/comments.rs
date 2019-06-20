use crate::api::Api;
use squs_models::{comments::*, db_conn::DbConn, posts::Post, Connection};
use serde::Serialize;
use rocket_contrib::json::Json;

#[derive(Serialize)]
pub struct Comm {
	avatar: String,
	author_name: String,
	author_fqn: String,
	content: String,
	date: String,
	cw: Option<String>,
	responses: Vec<Comm>,
}

impl Comm {
	fn from_comment(conn: &Connection, comm: Comment) -> Option<Comm> {
		let author = comm.get_author(conn).ok()?;
		Some(Comm {
			avatar: author.avatar_url.unwrap_or_default(),
			author_name: author.display_name.clone(),
			author_fqn: author.fqn.clone(),
			content: comm.content.to_string(),
			date: comm.creation_date.to_string(),
			cw: if comm.spoiler_text.is_empty() { None } else { Some(comm.spoiler_text.clone()) },
			responses: comm.get_responses(conn).ok()?.into_iter().filter_map(|c| Comm::from_comment(conn, c)).collect()
		})
	}
}

#[get("/comments?<article>")]
pub fn for_article(article: String, conn: DbConn) -> Api<Vec<Comm>> {
	let slug = article.replace("http://www.", "")
	    .replace("https://www.", "")
	    .replace("https://", "")
	    .replace("http://", "")
	    .replace("/", "");
	let post = Post::find_by_slug(&conn, &slug)?;
	let comments = Comment::list_by_post(&conn, post.id)?;
	Ok(Json(comments.into_iter()
		.filter_map(|c| Comm::from_comment(&*conn, c))
		.collect()
	))
}
