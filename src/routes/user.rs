use activitypub::collection::OrderedCollection;
use rocket::{
    http::Cookies,
    request::LenientForm,
    response::{status, Flash, Redirect},
};
use rocket_i18n::I18n;
use serde_json;
use std::{borrow::Cow, collections::HashMap};
use validator::{Validate, ValidationError, ValidationErrors};

use inbox;
use squs_common::activity_pub::{broadcast, ActivityStream, ApRequest, Id};
use squs_models::{
    db_conn::DbConn,
    headers::Headers,
    instance::Instance,
    users::*,
    Error, PlumeRocket,
};
use routes::{errors::ErrorPage, Page};
use template_utils::{IntoContext, Ructe};

#[get("/followers?<page>", rank = 2)]
pub fn followers(
    page: Option<Page>,
    rockets: PlumeRocket,
    user: User,
) -> Result<Ructe, ErrorPage> {
    let conn = &*rockets.conn;
    let page = page.unwrap_or_default();
    let followers_count = user.count_followers(&*conn)?;

    Ok(render!(users::followers(
        &rockets.to_context(),
        user.clone(),
        user.get_followers_page(&*conn, page.limits())?,
        page.0,
        Page::total(followers_count as i32)
    )))
}

#[get("/@/<name>", rank = 1)]
pub fn activity_details(
    name: String,
    rockets: PlumeRocket,
    _ap: ApRequest,
) -> Option<ActivityStream<CustomPerson>> {
    let user = User::find_by_fqn(&rockets, &name).ok()?;
    Some(ActivityStream::new(user.to_activity().ok()?))
}

#[get("/users/new")]
pub fn new(rockets: PlumeRocket) -> Result<Ructe, ErrorPage> {
    Ok(render!(users::new(
        &rockets.to_context(),
        Instance::get_local()?.open_registrations,
        &NewUserForm::default(),
        ValidationErrors::default()
    )))
}

#[get("/settings")]
pub fn edit(user: User, rockets: PlumeRocket) -> Ructe {
    render!(users::edit(
        &rockets.to_context(),
        UpdateUserForm {
            display_name: user.display_name.clone(),
            email: user.email.clone().unwrap_or_default(),
            summary: user.summary,
        },
        ValidationErrors::default()
    ))
}

#[derive(FromForm)]
pub struct UpdateUserForm {
    pub display_name: String,
    pub email: String,
    pub summary: String,
}

#[put("/settings", data = "<form>")]
pub fn update(
    conn: DbConn,
    user: User,
    form: LenientForm<UpdateUserForm>,
    intl: I18n,
) -> Result<Flash<Redirect>, ErrorPage> {
    user.update(
        &*conn,
        if !form.display_name.is_empty() {
            form.display_name.clone()
        } else {
            user.display_name.clone()
        },
        if !form.email.is_empty() {
            form.email.clone()
        } else {
            user.email.clone().unwrap_or_default()
        },
        if !form.summary.is_empty() {
            form.summary.clone()
        } else {
            user.summary.to_string()
        },
    )?;
    Ok(Flash::success(
        Redirect::to(uri!(super::instance::index)),
        i18n!(intl.catalog, "Your profile has been updated."),
    ))
}

#[post("/delete")]
pub fn delete(
    user: User,
    mut cookies: Cookies,
    rockets: PlumeRocket,
) -> Result<Flash<Redirect>, ErrorPage> {
    user.delete(&*rockets.conn)?;

    let target = User::one_by_instance(&*rockets.conn)?;
    let delete_act = user.delete_activity(&*rockets.conn)?;
    rockets
        .worker
        .execute(move || broadcast(&user, delete_act, target));

    if let Some(cookie) = cookies.get_private(AUTH_COOKIE) {
        cookies.remove_private(cookie);
    }

    Ok(Flash::success(
        Redirect::to(uri!(super::instance::index)),
        i18n!(rockets.intl.catalog, "Your account has been deleted."),
    ))
}

#[derive(Default, FromForm, Validate)]
#[validate(schema(
    function = "passwords_match",
    skip_on_field_errors = "false",
    message = "Passwords are not matching"
))]
pub struct NewUserForm {
    #[validate(
        length(min = "1", message = "Username can't be empty"),
        custom(
            function = "validate_username",
            message = "User name is not allowed to contain any of < > & @ ' or \""
        )
    )]
    pub username: String,
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = "8", message = "Password should be at least 8 characters long"))]
    pub password: String,
    #[validate(length(min = "8", message = "Password should be at least 8 characters long"))]
    pub password_confirmation: String,
}

pub fn passwords_match(form: &NewUserForm) -> Result<(), ValidationError> {
    if form.password != form.password_confirmation {
        Err(ValidationError::new("password_match"))
    } else {
        Ok(())
    }
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.contains(&['<', '>', '&', '@', '\'', '"', ' ', '\n', '\t'][..]) {
        Err(ValidationError::new("username_illegal_char"))
    } else {
        Ok(())
    }
}

fn to_validation(_: Error) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    errors.add(
        "",
        ValidationError {
            code: Cow::from("server_error"),
            message: Some(Cow::from("An unknown error occured")),
            params: HashMap::new(),
        },
    );
    errors
}

#[post("/users/new", data = "<form>")]
pub fn create(
    form: LenientForm<NewUserForm>,
    rockets: PlumeRocket,
) -> Result<Flash<Redirect>, Ructe> {
    let conn = &*rockets.conn;
    if !Instance::get_local()
        .map(|i| i.open_registrations)
        .unwrap_or(true)
    {
        return Ok(Flash::error(
            Redirect::to(uri!(new)),
            i18n!(
                rockets.intl.catalog,
                "Registrations are closed on this instance."
            ),
        )); // Actually, it is an error
    }

    let mut form = form.into_inner();
    form.username = form.username.trim().to_owned();
    form.email = form.email.trim().to_owned();
    form.validate()
        .and_then(|_| {
            NewUser::new_local(
                conn,
                form.username.to_string(),
                form.username.to_string(),
                false,
                "",
                form.email.to_string(),
                User::hash_pass(&form.password).map_err(to_validation)?,
            )
            .map_err(to_validation)?;
            Ok(Flash::success(
                Redirect::to(uri!(super::session::new: m = _)),
                i18n!(
                    rockets.intl.catalog,
                    "Your account has been created. Now you just need to log in, before you can use it."
                ),
            ))
        })
        .map_err(|err| {
            render!(users::new(
                &rockets.to_context(),
                Instance::get_local()
                    .map(|i| i.open_registrations)
                    .unwrap_or(true),
                &form,
                err
            ))
        })
}

#[get("/@/<name>/outbox")]
pub fn outbox(name: String, rockets: PlumeRocket) -> Option<ActivityStream<OrderedCollection>> {
    let user = User::find_by_fqn(&rockets, &name).ok()?;
    user.outbox(&*rockets.conn).ok()
}

#[post("/@/<name>/inbox", data = "<data>")]
pub fn inbox(
    name: String,
    data: inbox::SignedJson<serde_json::Value>,
    headers: Headers,
    rockets: PlumeRocket,
) -> Result<String, status::BadRequest<&'static str>> {
    User::find_by_fqn(&rockets, &name).map_err(|_| status::BadRequest(Some("User not found")))?;
    inbox::handle_incoming(rockets, data, headers)
}

#[get("/@/<name>/followers", rank = 1)]
pub fn ap_followers(
    name: String,
    rockets: PlumeRocket,
    _ap: ApRequest,
) -> Option<ActivityStream<OrderedCollection>> {
    let user = User::find_by_fqn(&rockets, &name).ok()?;
    let followers = user
        .get_followers(&*rockets.conn)
        .ok()?
        .into_iter()
        .map(|f| Id::new(f.ap_id))
        .collect::<Vec<Id>>();

    let mut coll = OrderedCollection::default();
    coll.object_props
        .set_id_string(user.followers_endpoint)
        .ok()?;
    coll.collection_props
        .set_total_items_u64(followers.len() as u64)
        .ok()?;
    coll.collection_props.set_items_link_vec(followers).ok()?;
    Some(ActivityStream::new(coll))
}
