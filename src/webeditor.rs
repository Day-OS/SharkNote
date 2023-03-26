use html_editor::{operation::{Editable, Selector, Htmlifiable}, Node};
use rocket::{response::content::RawHtml, serde::__private::doc};
use std::fs::File;


pub fn get_editor(defaultdir: &str, page_name: &str, document_name: String) -> RawHtml<String>{

    let mut document = html_editor::parse(
        std::str::from_utf8(std::fs::read("/home/ubuntu/DEV/daytheipc-com/public/webeditor.html").unwrap().as_slice()).unwrap()
        ).expect("Should load editor page");
 
    //html_editor::parse(include_str!("../public/webeditor.html")).expect("Should load editor page");

    //PAGE TITLE
    let mut page_title: String = "Editing page: ".to_string();
    page_title.push_str(page_name.clone());
    page_title.push_str(" -> ");
    page_title.push_str(&document_name.clone());
    document.insert_to(&Selector::from("title"), Node::Text(page_title));

    document.insert_to(&Selector::from("div#original-document-content"), Node::Text("# aaa \n ## a \n - a \n ()[]".to_string()));
    
    RawHtml(document.html())
}