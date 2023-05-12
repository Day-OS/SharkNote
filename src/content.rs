use rocket::{get, fs::NamedFile};
use crate::utils::get_program_files_location;

#[get("/<page_name>/<content_dir>/<content_name>")]
pub async fn get_content(page_name: &str, content_dir: String, content_name: &str) -> Option<NamedFile>{
    let mut page =  get_program_files_location();
    
    if content_dir.eq("public") {
        page.push_str("pages/");
        page.push_str(page_name);page.push('/');
        page.push_str("public/");
        page.push_str(content_name);
    }
    else if content_dir.eq("global-public") {
        page.push_str("public/");
        page.push_str(content_name);
    }
    else{}
    println!("{}",&page);
    NamedFile::open(page).await.ok()
}