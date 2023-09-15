use std::path::{PathBuf, Path};
use crate::{pages, database};
use rocket::{get, fs::NamedFile};


//GETS CONTENT FROM A PAGE TO LOAD TO THE EDITOR
#[get("/content/<path..>")]
pub async fn get_content(path: PathBuf) -> Option<NamedFile>{
    let mut page_dir = database::get_program_files_location();
    page_dir.push_str(pages::PAGE_DIR);
    let path = Path::new(&page_dir).join(path);
    NamedFile::open(path).await.ok()
}