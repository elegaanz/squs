use squs_models::{notifications::Notification, users::User, PlumeRocket};
use routes::{errors::ErrorPage, Page};
use template_utils::{IntoContext, Ructe};

#[get("/notifications?<page>")]
pub fn notifications(
    user: User,
    page: Option<Page>,
    rockets: PlumeRocket,
) -> Result<Ructe, ErrorPage> {
    let page = page.unwrap_or_default();
    Ok(render!(notifications::index(
        &rockets.to_context(),
        Notification::page_for_user(&*rockets.conn, &user, page.limits())?,
        page.0,
        Page::total(Notification::count_for_user(&*rockets.conn, &user)? as i32)
    )))
}
