#![warn(clippy::too_many_arguments)]
use rocket::{
    http::{
        hyper::header::{CacheControl, CacheDirective},
        uri::{FromUriParam, Query},
        RawStr, Status,
    },
    request::{self, FromFormValue, FromRequest, Request},
    response::{Flash, NamedFile, Redirect},
    Outcome,
};
use std::path::{Path, PathBuf};
use template_utils::Ructe;

const ITEMS_PER_PAGE: i32 = 12;

/// Special return type used for routes that "cannot fail", and instead
/// `Redirect`, or `Flash<Redirect>`, when we cannot deliver a `Ructe` Response
#[allow(clippy::large_enum_variant)]
#[derive(Responder)]
pub enum RespondOrRedirect {
    Response(Ructe),
    FlashResponse(Flash<Ructe>),
    Redirect(Redirect),
    FlashRedirect(Flash<Redirect>),
}

impl From<Ructe> for RespondOrRedirect {
    fn from(response: Ructe) -> Self {
        RespondOrRedirect::Response(response)
    }
}

impl From<Flash<Ructe>> for RespondOrRedirect {
    fn from(response: Flash<Ructe>) -> Self {
        RespondOrRedirect::FlashResponse(response)
    }
}

impl From<Redirect> for RespondOrRedirect {
    fn from(redirect: Redirect) -> Self {
        RespondOrRedirect::Redirect(redirect)
    }
}

impl From<Flash<Redirect>> for RespondOrRedirect {
    fn from(redirect: Flash<Redirect>) -> Self {
        RespondOrRedirect::FlashRedirect(redirect)
    }
}

#[derive(Shrinkwrap, Copy, Clone, UriDisplayQuery)]
pub struct Page(i32);

impl<'v> FromFormValue<'v> for Page {
    type Error = &'v RawStr;
    fn from_form_value(form_value: &'v RawStr) -> Result<Page, &'v RawStr> {
        match form_value.parse::<i32>() {
            Ok(page) => Ok(Page(page)),
            _ => Err(form_value),
        }
    }
}

impl FromUriParam<Query, Option<Page>> for Page {
    type Target = Page;

    fn from_uri_param(val: Option<Page>) -> Page {
        val.unwrap_or_default()
    }
}

impl Page {
    /// Computes the total number of pages needed to display n_items
    pub fn total(n_items: i32) -> i32 {
        if n_items % ITEMS_PER_PAGE == 0 {
            n_items / ITEMS_PER_PAGE
        } else {
            (n_items / ITEMS_PER_PAGE) + 1
        }
    }

    pub fn limits(self) -> (i32, i32) {
        ((self.0 - 1) * ITEMS_PER_PAGE, self.0 * ITEMS_PER_PAGE)
    }
}

#[derive(Shrinkwrap)]
pub struct ContentLen(pub u64);

impl<'a, 'r> FromRequest<'a, 'r> for ContentLen {
    type Error = ();

    fn from_request(r: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        match r.limits().get("forms") {
            Some(l) => Outcome::Success(ContentLen(l)),
            None => Outcome::Failure((Status::InternalServerError, ())),
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page(1)
    }
}

pub mod comments;
pub mod errors;
pub mod instance;
pub mod likes;
pub mod notifications;
pub mod posts;
pub mod reshares;
pub mod session;
pub mod user;
pub mod well_known;

#[derive(Responder)]
#[response()]
pub struct CachedFile {
    inner: NamedFile,
    cache_control: CacheControl,
}

#[get("/static/cached/<_build_id>/<file..>", rank = 2)]
pub fn plume_static_files(file: PathBuf, _build_id: &RawStr) -> Option<CachedFile> {
    static_files(file)
}

#[get("/static/<file..>", rank = 3)]
pub fn static_files(file: PathBuf) -> Option<CachedFile> {
    NamedFile::open(Path::new("static/").join(file))
        .ok()
        .map(|f| CachedFile {
            inner: f,
            cache_control: CacheControl(vec![CacheDirective::MaxAge(60 * 60 * 24 * 30)]),
        })
}
