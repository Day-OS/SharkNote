use rocket::{get, response::content::RawHtml};
use rocket_db_pools::Connection;

//pub mod database;
pub mod page;

#[get("/<page_id>/<path>?preview=true")]
pub fn preview(
    connection: Connection<crate::DATABASE>,
    page_id: String,
    path: String,
) -> RawHtml<String> {
    //Page::get(connection, page_id);
    RawHtml(r#"<meta property="og:type" content="website">
    <meta property="og:site_name" content="DayTheIPC">
    <meta property="og:title" content="EU TENHO PROBLEMA MENTAL">
    <meta property="og:description" content="leonnn">
    <meta property="og:image" content="https://media.tenor.com/yupzrpi446EAAAAC/re4-leon-kennedy.gif">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:site" content="@discord">
    <meta name="twitter:creator" content="@yourtwitter">"#.into())
}
