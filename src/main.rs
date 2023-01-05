#[macro_use] extern crate rocket;
use rocket::response::content::{RawHtml};
use std::fs;
use std::str;
static DEFAULT_DIR: &'static str = "/home/ubuntu/pages/";

#[get("/c/<pagename>")]
fn get_content(pagename: &str) {//-> RawHtml<String> {
    
}

#[get("/p/<pagename>")]
fn get_page(pagename: &str) -> RawHtml<String> {
    //Setups MarkDownIt
    let md_parser = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md_parser);
    markdown_it::plugins::extra::add(md_parser);
    markdown_it::plugins::html::add(md_parser);


    //GETS INDEX.HTML
    let mut default_html: String = "".to_string();
    default_html.push_str(DEFAULT_DIR); default_html.push_str("index.html");
    let file = fs::read(default_html).expect("Should load index.html");
    let index_html = str::from_utf8(&file).clone().unwrap();
    let split = index_html.split("CANVAS");
    let vec_index_html: Vec<&str> = split.collect();

    //Gets URL page parameter, loads md file and parses it into HTML
    let mut page = DEFAULT_DIR.clone().to_owned();
    page.push_str(pagename);
    if !page.ends_with(".md") || !page.ends_with(".html") {page.push_str(".md");}
    let file = fs::read(page).expect("Should load file");
    let file_str = str::from_utf8(&file).unwrap();
    let md_parsed: String  = md_parser.parse(file_str).render();
   
    let mut final_html = vec_index_html[0].to_string();
    final_html.push_str(&md_parsed);
    final_html.push_str(vec_index_html[1]);



    print!("{}", final_html);
    RawHtml(final_html)
}

#[launch]
fn rocket() -> _ {
    let figment = rocket::Config::figment()
        .merge(("port", 8000));
    rocket::custom(figment).mount("/", routes![get_page])
}
