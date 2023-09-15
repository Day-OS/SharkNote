use rocket::{catch, response::content::RawHtml};

#[catch(404)]
pub fn not_found() -> RawHtml<String> {
    RawHtml("página não encontrada!\n 404 WIP".to_string())
}
