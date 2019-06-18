use activitypub::activity::*;
use serde_json;

use crate::{
    comments::Comment,
    follows, likes,
    posts::Post,
    reshares::Reshare,
    users::User,
    Error, PlumeRocket,
};
use squs_common::activity_pub::inbox::Inbox;

macro_rules! impl_into_inbox_result {
    ( $( $t:ty => $variant:ident ),+ ) => {
        $(
            impl From<$t> for InboxResult {
                fn from(x: $t) -> InboxResult {
                    InboxResult::$variant(x)
                }
            }
        )+
    }
}

pub enum InboxResult {
    Commented(Comment),
    Followed(follows::Follow),
    Liked(likes::Like),
    Other,
    Post(Post),
    Reshared(Reshare),
}

impl From<()> for InboxResult {
    fn from(_: ()) -> InboxResult {
        InboxResult::Other
    }
}

impl_into_inbox_result! {
    Comment => Commented,
    follows::Follow => Followed,
    likes::Like => Liked,
    Post => Post,
    Reshare => Reshared
}

pub fn inbox(ctx: &PlumeRocket, act: serde_json::Value) -> Result<InboxResult, Error> {
    Inbox::handle(ctx, act)
        .with::<User, Announce, Comment>()
        .with::<User, Create, Comment>()
        .with::<User, Delete, Comment>()
        .with::<User, Delete, User>()
        .with::<User, Follow, User>()
        .with::<User, Like, Comment>()
        .with::<User, Undo, Reshare>()
        .with::<User, Undo, follows::Follow>()
        .with::<User, Undo, likes::Like>()
        .done()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::InboxResult;
    use crate::blogs::tests::fill_database as blog_fill_db;
    use crate::safe_string::SafeString;
    use crate::tests::rockets;
    use crate::PlumeRocket;
    use diesel::Connection;

    pub fn fill_database(
        rockets: &PlumeRocket,
    ) -> (
        Vec<crate::posts::Post>,
        Vec<crate::users::User>,
        Vec<crate::blogs::Blog>,
    ) {
        use crate::post_authors::*;
        use crate::posts::*;

        let (users, blogs) = blog_fill_db(&rockets.conn);
        let post = Post::insert(
            &rockets.conn,
            NewPost {
                blog_id: blogs[0].id,
                slug: "testing".to_owned(),
                title: "Testing".to_owned(),
                content: crate::safe_string::SafeString::new("Hello"),
                published: true,
                license: "WTFPL".to_owned(),
                creation_date: None,
                ap_id: format!("https://plu.me/~/{}/testing", blogs[0].actor_id),
                subtitle: String::new(),
                source: String::new(),
                cover_id: None,
            },
            &rockets.searcher,
        )
        .unwrap();

        PostAuthor::insert(
            &rockets.conn,
            NewPostAuthor {
                post_id: post.id,
                author_id: users[0].id,
            },
        )
        .unwrap();

        (vec![post], users, blogs)
    }

    #[test]
    fn announce_post() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (posts, users, _) = fill_database(&r);
            let act = json!({
                "id": "https://plu.me/announce/1",
                "actor": users[0].ap_id,
                "object": posts[0].ap_id,
                "type": "Announce",
            });

            match super::inbox(&r, act).unwrap() {
                super::InboxResult::Reshared(r) => {
                    assert_eq!(r.post_id, posts[0].id);
                    assert_eq!(r.user_id, users[0].id);
                    assert_eq!(r.ap_id, "https://plu.me/announce/1".to_owned());
                }
                _ => panic!("Unexpected result"),
            };

            Ok(())
        });
    }

    #[test]
    fn create_comment() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (posts, users, _) = fill_database(&r);
            let act = json!({
                "id": "https://plu.me/comment/1/activity",
                "actor": users[0].ap_id,
                "object": {
                    "type": "Note",
                    "id": "https://plu.me/comment/1",
                    "attributedTo": users[0].ap_id,
                    "inReplyTo": posts[0].ap_id,
                    "content": "Hello.",
                    "to": [squs_common::activity_pub::PUBLIC_VISIBILITY]
                },
                "type": "Create",
            });

            match super::inbox(&r, act).unwrap() {
                super::InboxResult::Commented(c) => {
                    assert_eq!(c.author_id, users[0].id);
                    assert_eq!(c.post_id, posts[0].id);
                    assert_eq!(c.in_response_to_id, None);
                    assert_eq!(c.content, SafeString::new("Hello."));
                    assert!(c.public_visibility);
                }
                _ => panic!("Unexpected result"),
            };

            Ok(())
        });
    }

    #[test]
    fn create_post() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (_, users, blogs) = fill_database(&r);
            let act = json!({
                "id": "https://plu.me/comment/1/activity",
                "actor": users[0].ap_id,
                "object": {
                    "type": "Article",
                    "id": "https://plu.me/~/Blog/my-article",
                    "attributedTo": [users[0].ap_id, blogs[0].ap_id],
                    "content": "Hello.",
                    "name": "My Article",
                    "summary": "Bye.",
                    "source": {
                        "content": "Hello.",
                        "mediaType": "text/markdown"
                    },
                    "published": "2014-12-12T12:12:12Z",
                    "to": [squs_common::activity_pub::PUBLIC_VISIBILITY]
                },
                "type": "Create",
            });

            match super::inbox(&r, act).unwrap() {
                super::InboxResult::Post(p) => {
                    assert!(p.is_author(conn, users[0].id).unwrap());
                    assert_eq!(p.source, "Hello.".to_owned());
                    assert_eq!(p.blog_id, blogs[0].id);
                    assert_eq!(p.content, SafeString::new("Hello."));
                    assert_eq!(p.subtitle, "Bye.".to_owned());
                    assert_eq!(p.title, "My Article".to_owned());
                }
                _ => panic!("Unexpected result"),
            };

            Ok(())
        });
    }

    #[test]
    fn delete_comment() {
        use crate::comments::*;

        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (posts, users, _) = fill_database(&r);
            Comment::insert(
                conn,
                NewComment {
                    content: SafeString::new("My comment"),
                    in_response_to_id: None,
                    post_id: posts[0].id,
                    author_id: users[0].id,
                    ap_id: Some("https://plu.me/comment/1".to_owned()),
                    sensitive: false,
                    spoiler_text: "spoiler".to_owned(),
                    public_visibility: true,
                },
            )
            .unwrap();

            let fail_act = json!({
                "id": "https://plu.me/comment/1/delete",
                "actor": users[1].ap_id, // Not the author of the comment, it should fail
                "object": "https://plu.me/comment/1",
                "type": "Delete",
            });
            assert!(super::inbox(&r, fail_act).is_err());

            let ok_act = json!({
                "id": "https://plu.me/comment/1/delete",
                "actor": users[0].ap_id,
                "object": "https://plu.me/comment/1",
                "type": "Delete",
            });
            assert!(super::inbox(&r, ok_act).is_ok());

            Ok(())
        });
    }

    #[test]
    fn delete_post() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (posts, users, _) = fill_database(&r);

            let fail_act = json!({
                "id": "https://plu.me/comment/1/delete",
                "actor": users[1].ap_id, // Not the author of the post, it should fail
                "object": posts[0].ap_id,
                "type": "Delete",
            });
            assert!(super::inbox(&r, fail_act).is_err());

            let ok_act = json!({
                "id": "https://plu.me/comment/1/delete",
                "actor": users[0].ap_id,
                "object": posts[0].ap_id,
                "type": "Delete",
            });
            assert!(super::inbox(&r, ok_act).is_ok());

            Ok(())
        });
    }

    #[test]
    fn delete_user() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (_, users, _) = fill_database(&r);

            let fail_act = json!({
                "id": "https://plu.me/@/Admin#delete",
                "actor": users[1].ap_id, // Not the same account
                "object": users[0].ap_id,
                "type": "Delete",
            });
            assert!(super::inbox(&r, fail_act).is_err());

            let ok_act = json!({
                "id": "https://plu.me/@/Admin#delete",
                "actor": users[0].ap_id,
                "object": users[0].ap_id,
                "type": "Delete",
            });
            assert!(super::inbox(&r, ok_act).is_ok());
            assert!(crate::users::User::get(conn, users[0].id).is_err());

            Ok(())
        });
    }

    #[test]
    fn follow() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (_, users, _) = fill_database(&r);

            let act = json!({
                "id": "https://plu.me/follow/1",
                "actor": users[0].ap_id,
                "object": users[1].ap_id,
                "type": "Follow",
            });
            match super::inbox(&r, act).unwrap() {
                InboxResult::Followed(f) => {
                    assert_eq!(f.follower_id, users[0].id);
                    assert_eq!(f.following_id, users[1].id);
                    assert_eq!(f.ap_id, "https://plu.me/follow/1".to_owned());
                }
                _ => panic!("Unexpected result"),
            }

            Ok(())
        });
    }

    #[test]
    fn like() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (posts, users, _) = fill_database(&r);

            let act = json!({
                "id": "https://plu.me/like/1",
                "actor": users[1].ap_id,
                "object": posts[0].ap_id,
                "type": "Like",
            });
            match super::inbox(&r, act).unwrap() {
                InboxResult::Liked(l) => {
                    assert_eq!(l.user_id, users[1].id);
                    assert_eq!(l.post_id, posts[0].id);
                    assert_eq!(l.ap_id, "https://plu.me/like/1".to_owned());
                }
                _ => panic!("Unexpected result"),
            }

            Ok(())
        });
    }

    #[test]
    fn undo_reshare() {
        use crate::reshares::*;

        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (posts, users, _) = fill_database(&r);

            let announce = Reshare::insert(
                conn,
                NewReshare {
                    post_id: posts[0].id,
                    user_id: users[1].id,
                    ap_id: "https://plu.me/announce/1".to_owned(),
                },
            )
            .unwrap();

            let fail_act = json!({
                "id": "https://plu.me/undo/1",
                "actor": users[0].ap_id,
                "object": announce.ap_id,
                "type": "Undo",
            });
            assert!(super::inbox(&r, fail_act).is_err());

            let ok_act = json!({
                "id": "https://plu.me/undo/1",
                "actor": users[1].ap_id,
                "object": announce.ap_id,
                "type": "Undo",
            });
            assert!(super::inbox(&r, ok_act).is_ok());

            Ok(())
        });
    }

    #[test]
    fn undo_follow() {
        use crate::follows::*;

        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (_, users, _) = fill_database(&r);

            let follow = Follow::insert(
                conn,
                NewFollow {
                    follower_id: users[0].id,
                    following_id: users[1].id,
                    ap_id: "https://plu.me/follow/1".to_owned(),
                },
            )
            .unwrap();

            let fail_act = json!({
                "id": "https://plu.me/undo/1",
                "actor": users[2].ap_id,
                "object": follow.ap_id,
                "type": "Undo",
            });
            assert!(super::inbox(&r, fail_act).is_err());

            let ok_act = json!({
                "id": "https://plu.me/undo/1",
                "actor": users[0].ap_id,
                "object": follow.ap_id,
                "type": "Undo",
            });
            assert!(super::inbox(&r, ok_act).is_ok());

            Ok(())
        });
    }

    #[test]
    fn undo_like() {
        use crate::likes::*;

        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (posts, users, _) = fill_database(&r);

            let like = Like::insert(
                conn,
                NewLike {
                    post_id: posts[0].id,
                    user_id: users[1].id,
                    ap_id: "https://plu.me/like/1".to_owned(),
                },
            )
            .unwrap();

            let fail_act = json!({
                "id": "https://plu.me/undo/1",
                "actor": users[0].ap_id,
                "object": like.ap_id,
                "type": "Undo",
            });
            assert!(super::inbox(&r, fail_act).is_err());

            let ok_act = json!({
                "id": "https://plu.me/undo/1",
                "actor": users[1].ap_id,
                "object": like.ap_id,
                "type": "Undo",
            });
            assert!(super::inbox(&r, ok_act).is_ok());

            Ok(())
        });
    }

    #[test]
    fn update_post() {
        let r = rockets();
        let conn = &*r.conn;
        conn.test_transaction::<_, (), _>(|| {
            let (posts, users, _) = fill_database(&r);

            let act = json!({
                "id": "https://plu.me/update/1",
                "actor": users[0].ap_id,
                "object": {
                    "type": "Article",
                    "id": posts[0].ap_id,
                    "name": "Mia Artikolo",
                    "summary": "Jes, mi parolas esperanton nun",
                    "content": "<b>Saluton</b>, mi skribas testojn",
                    "source": {
                        "mediaType": "text/markdown",
                        "content": "**Saluton**, mi skribas testojn"
                    },
                },
                "type": "Update",
            });

            super::inbox(&r, act).unwrap();

            Ok(())
        });
    }
}
