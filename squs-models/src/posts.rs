use activitypub::{
    activity::{Create, Delete, Update},
    link,
    object::{Article, Tombstone},
    CustomObject,
};
use chrono::{NaiveDateTime, TimeZone, Utc};
use diesel::{self, ExpressionMethods, QueryDsl, RunQueryDsl, SaveChangesDsl};
use serde_json;
use std::collections::HashSet;

use mentions::Mention;
use squs_common::activity_pub::{Id, IntoId, Licensed, PUBLIC_VISIBILITY};
use safe_string::SafeString;
use schema::posts;
use users::User;
use {Connection, Error, Result, CONFIG};

pub type LicensedArticle = CustomObject<Licensed, Article>;

#[derive(Queryable, Identifiable, Clone, AsChangeset)]
#[changeset_options(treat_none_as_null = "true")]
pub struct Post {
    pub id: i32,
    pub url: String,
    pub author_id: i32,
    pub title: String,
    pub content: SafeString,
    pub license: String,
    pub creation_date: NaiveDateTime,
    pub ap_id: String,
    pub subtitle: String,
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    pub url: String,
    pub author_id: i32,
    pub title: String,
    pub content: SafeString,
    pub license: String,
    pub ap_id: String,
    pub subtitle: String,
}

impl Post {
    get!(posts);
    find_by!(posts, find_by_ap_id, ap_id as &str);
    find_by!(posts, find_by_url, url as &str);
    insert!(posts, NewPost, |new, conn| {
        if new.ap_id.is_empty() {
            new.ap_id = format!("https://{}/p/{}", CONFIG.base_url, new.id);
            let _: Post = new.save_changes(conn)?;
        }
        Ok(new)
    });

    pub fn update(&self, conn: &Connection) -> Result<Self> {
        diesel::update(self).set(self).execute(conn)?;
        Post::get(conn, self.id)
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        for m in Mention::list_for_post(&conn, self.id)? {
            m.delete(conn)?;
        }
        diesel::delete(self).execute(conn)?;
        Ok(())
    }

    pub fn list_filtered(
        conn: &Connection,
        title: Option<String>,
        subtitle: Option<String>,
        content: Option<String>,
    ) -> Result<Vec<Post>> {
        let mut query = posts::table.into_boxed();
        if let Some(title) = title {
            query = query.filter(posts::title.eq(title));
        }
        if let Some(subtitle) = subtitle {
            query = query.filter(posts::subtitle.eq(subtitle));
        }
        if let Some(content) = content {
            query = query.filter(posts::content.eq(content));
        }

        query.get_results::<Post>(conn).map_err(Error::from)
    }

    pub fn get_recents_for_author(
        conn: &Connection,
        author: &User,
        limit: i64,
    ) -> Result<Vec<Post>> {
        posts::table
            .filter(posts::author_id.eq(author.id))
            .order(posts::creation_date.desc())
            .limit(limit)
            .load::<Post>(conn)
            .map_err(Error::from)
    }

    pub fn get_author(&self, conn: &Connection) -> Result<User> {
        User::get(conn, self.author_id)
    }

    pub fn get_receivers_urls(&self, conn: &Connection) -> Result<Vec<String>> {
        Ok(self
            .get_author(conn)?
            .get_followers(conn)?
            .into_iter()
            .map(|x| x.ap_id)
            .collect()
        )
    }

    pub fn to_activity(&self, conn: &Connection) -> Result<LicensedArticle> {
        let cc = self.get_receivers_urls(conn)?;
        let to = vec![PUBLIC_VISIBILITY.to_string()];

        let mentions_json = Mention::list_for_post(conn, self.id)?
            .into_iter()
            .map(|m| json!(m.to_activity(conn).ok()))
            .collect::<Vec<serde_json::Value>>();

        let mut article = Article::default();
        article.object_props.set_name_string(self.title.clone())?;
        article.object_props.set_id_string(self.ap_id.clone())?;

        let authors = vec![Id::new(User::get(conn, self.author_id)?.ap_id)];
        article
            .object_props
            .set_attributed_to_link_vec::<Id>(authors)?;
        article
            .object_props
            .set_content_string(self.content.get().clone())?;
        article
            .object_props
            .set_published_utctime(Utc.from_utc_datetime(&self.creation_date))?;
        article
            .object_props
            .set_summary_string(self.subtitle.clone())?;
        article.object_props.tag = Some(json!(mentions_json));

        article.object_props.set_url_string(self.url.clone())?;
        article
            .object_props
            .set_to_link_vec::<Id>(to.into_iter().map(Id::new).collect())?;
        article
            .object_props
            .set_cc_link_vec::<Id>(cc.into_iter().map(Id::new).collect())?;
        let mut license = Licensed::default();
        license.set_license_string(self.license.clone())?;
        Ok(LicensedArticle::new(article, license))
    }

    pub fn create_activity(&self, conn: &Connection) -> Result<Create> {
        let article = self.to_activity(conn)?;
        let mut act = Create::default();
        act.object_props
            .set_id_string(format!("{}activity", self.ap_id))?;
        act.object_props
            .set_to_link_vec::<Id>(article.object.object_props.to_link_vec()?)?;
        act.object_props
            .set_cc_link_vec::<Id>(article.object.object_props.cc_link_vec()?)?;
        act.create_props
            .set_actor_link(Id::new(User::get(conn, self.author_id)?.ap_id))?;
        act.create_props.set_object_object(article)?;
        Ok(act)
    }

    pub fn update_activity(&self, conn: &Connection) -> Result<Update> {
        let article = self.to_activity(conn)?;
        let mut act = Update::default();
        act.object_props.set_id_string(format!(
            "{}/update-{}",
            self.ap_id,
            Utc::now().timestamp()
        ))?;
        act.object_props
            .set_to_link_vec::<Id>(article.object.object_props.to_link_vec()?)?;
        act.object_props
            .set_cc_link_vec::<Id>(article.object.object_props.cc_link_vec()?)?;
        act.update_props
            .set_actor_link(Id::new(User::get(conn, self.author_id)?.ap_id))?;
        act.update_props.set_object_object(article)?;
        Ok(act)
    }

    pub fn update_mentions(&self, conn: &Connection, mentions: Vec<link::Mention>) -> Result<()> {
        let mentions = mentions
            .into_iter()
            .map(|m| {
                (
                    m.link_props
                        .href_string()
                        .ok()
                        .and_then(|ap_id| User::find_by_ap_id(conn, &ap_id).ok())
                        .map(|u| u.id),
                    m,
                )
            })
            .filter_map(|(id, m)| {
                if let Some(id) = id {
                    Some((m, id))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let old_mentions = Mention::list_for_post(&conn, self.id)?;
        let old_user_mentioned = old_mentions
            .iter()
            .map(|m| m.mentioned_id)
            .collect::<HashSet<_>>();
        for (m, id) in &mentions {
            if !old_user_mentioned.contains(&id) {
                Mention::from_activity(&*conn, &m, self.id, true, true)?;
            }
        }

        let new_mentions = mentions
            .into_iter()
            .map(|(_m, id)| id)
            .collect::<HashSet<_>>();
        for m in old_mentions
            .iter()
            .filter(|m| !new_mentions.contains(&m.mentioned_id))
        {
            m.delete(&conn)?;
        }
        Ok(())
    }

    pub fn build_delete(&self, conn: &Connection) -> Result<Delete> {
        let mut act = Delete::default();
        act.delete_props
            .set_actor_link(self.get_author(conn)?.into_id())?;

        let mut tombstone = Tombstone::default();
        tombstone.object_props.set_id_string(self.ap_id.clone())?;
        act.delete_props.set_object_object(tombstone)?;

        act.object_props
            .set_id_string(format!("{}#delete", self.ap_id))?;
        act.object_props
            .set_to_link_vec(vec![Id::new(PUBLIC_VISIBILITY)])?;
        Ok(act)
    }
}

impl IntoId for Post {
    fn into_id(self) -> Id {
        Id::new(self.ap_id.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbox::{inbox, tests::fill_database, InboxResult};
    use crate::safe_string::SafeString;
    use crate::tests::rockets;
    use diesel::Connection;

    // creates a post, get it's Create activity, delete the post,
    // "send" the Create to the inbox, and check it works
    #[test]
    fn self_federation() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (_, users, blogs) = fill_database(&r);
            let post = Post::insert(
                conn,
                NewPost {
                    blog_id: blogs[0].id,
                    slug: "yo".into(),
                    title: "Yo".into(),
                    content: SafeString::new("Hello"),
                    published: true,
                    license: "WTFPL".to_string(),
                    creation_date: None,
                    ap_id: String::new(), // automatically updated when inserting
                    subtitle: "Testing".into(),
                    source: "Hello".into(),
                    cover_id: None,
                },
                &r.searcher,
            )
            .unwrap();
            PostAuthor::insert(
                conn,
                NewPostAuthor {
                    post_id: post.id,
                    author_id: users[0].id,
                },
            )
            .unwrap();
            let create = post.create_activity(conn).unwrap();
            post.delete(conn, &r.searcher).unwrap();

            match inbox(&r, serde_json::to_value(create).unwrap()).unwrap() {
                InboxResult::Post(p) => {
                    assert!(p.is_author(conn, users[0].id).unwrap());
                    assert_eq!(p.source, "Hello".to_owned());
                    assert_eq!(p.blog_id, blogs[0].id);
                    assert_eq!(p.content, SafeString::new("Hello"));
                    assert_eq!(p.subtitle, "Testing".to_owned());
                    assert_eq!(p.title, "Yo".to_owned());
                }
                _ => panic!("Unexpected result"),
            };

            Ok(())
        });
    }

    #[test]
    fn licensed_article_serde() {
        let mut article = Article::default();
        article.object_props.set_id_string("Yo".into()).unwrap();
        let mut license = Licensed::default();
        license.set_license_string("WTFPL".into()).unwrap();
        let full_article = LicensedArticle::new(article, license);

        let json = serde_json::to_value(full_article).unwrap();
        let article_from_json: LicensedArticle = serde_json::from_value(json).unwrap();
        assert_eq!(
            "Yo",
            &article_from_json.object.object_props.id_string().unwrap()
        );
        assert_eq!(
            "WTFPL",
            &article_from_json.custom_props.license_string().unwrap()
        );
    }

    #[test]
    fn licensed_article_deserialization() {
        let json = json!({
            "type": "Article",
            "id": "https://plu.me/~/Blog/my-article",
            "attributedTo": ["https://plu.me/@/Admin", "https://plu.me/~/Blog"],
            "content": "Hello.",
            "name": "My Article",
            "summary": "Bye.",
            "source": {
                "content": "Hello.",
                "mediaType": "text/markdown"
            },
            "published": "2014-12-12T12:12:12Z",
            "to": [squs_common::activity_pub::PUBLIC_VISIBILITY]
        });
        let article: LicensedArticle = serde_json::from_value(json).unwrap();
        assert_eq!(
            "https://plu.me/~/Blog/my-article",
            &article.object.object_props.id_string().unwrap()
        );
    }
}
