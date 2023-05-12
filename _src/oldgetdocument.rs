use html_editor::{operation::*, Node};
use rocket::{fs::NamedFile, response::content::RawHtml};
use std::{str, fs};
use std::borrow::BorrowMut;
use crate::DEFAULT_DIR;
#[path ="table_of_contents.rs"]
mod table_of_contents;



#[get("/old/<page_name>/<document>")]
pub fn get_page(page_name: &str, document: &str) -> RawHtml<String> {
    //Setups MarkDownIt
    let md_parser = &mut markdown_it::MarkdownIt::new();
    markdown_it::plugins::cmark::add(md_parser);
    markdown_it::plugins::extra::add(md_parser);
    markdown_it::plugins::html::add(md_parser);


    //GETS INDEX.HTML
    let mut default_html: String = "".to_string();
    default_html.push_str(DEFAULT_DIR); default_html.push_str("pages/index.html");
    let file = fs::read(default_html).expect("Should load index.html");
    let index_html = str::from_utf8(&file).clone().unwrap();

    //Gets URL page parameter, loads md file and parses it into HTML
    let mut page = DEFAULT_DIR.clone().to_owned();
    page.push_str("pages/");
    page.push_str(page_name);
    page.push('/');
    page.push_str(document);
    if !page.ends_with(".md") || !page.ends_with(".html") {page.push_str(".md");}
    let file = fs::read(page).expect("Should load file");
    let file_str = str::from_utf8(&file).unwrap();
    let md_parsed: String  = md_parser.parse(file_str).render();
    let mut md_vector: Vec<Node> = html_editor::parse(&md_parsed).unwrap();//vec![Node::Text(md_parsed)];
    
    //Gets HTML, filters the body and create footer
    let mut dom = html_editor::parse(index_html).unwrap();
    let body_selec = Selector::from("body");


    //TABLE OF CONTENTS
    let mut toc: Vec<Node> = vec![];
    
    table_of_contents::add_table(md_vector.borrow_mut(), toc.borrow_mut());
    
    dom.insert_to(&body_selec, Node::new_element("div", vec![("id", "toc")], toc));
    dom.insert_to(&body_selec, Node::new_element("div", vec![("class","md")], md_vector));
    dom.insert_to(&body_selec, Node::new_element("footer", vec![], vec![Node::Text(r#"2022 DaytheIPC<sup>NOT A TM</sup> - <a href="https://srv.daytheipc.com/wtfpl.txt">WTFPL</a>"#.to_string())]));
    let final_html = dom.html();

    RawHtml(final_html)
}