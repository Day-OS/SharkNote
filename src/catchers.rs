use rocket::{catch};
use rocket_dyn_templates::{context, Template};

#[catch(404)]
pub fn not_found() -> Template {
    Template::render("error/not_found", context! {})
}
#[catch(401)]
pub fn not_authorized() -> Template {
    Template::render("error/not_authorized", context! {})
}
#[catch(500)]
pub fn internal_error() -> Template {
    Template::render("error/internal_error", context! {})
}
