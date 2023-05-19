use std::path::{PathBuf, Path};
use crate::{pages, utils};
use rocket::{get, fs::NamedFile};


#[get("/content/<path..>")]
pub async fn get_content(path: PathBuf) -> Option<NamedFile>{
    let mut page_dir = utils::get_program_files_location();
    page_dir.push_str(pages::PAGE_DIR);
    let path = Path::new(&page_dir).join(path);
    NamedFile::open(path).await.ok()
}