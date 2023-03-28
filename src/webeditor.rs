use html_editor::{operation::{Editable, Selector, Htmlifiable}, Node};
use rocket::{response::content::RawHtml, serde::__private::doc, http::{Cookie, CookieJar}};
use std::fs::File;

#[get("/editor")]
pub fn get_editor(user: CookieJar) -> RawHtml<String>{
    let login = std::str::from_utf8(
        std::fs::read("/home/ubuntu/DEV/daytheipc-com/public/login.html").unwrap().as_slice()
        ).unwrap().to_string();
    let webeditor = std::str::from_utf8(
        std::fs::read("/home/ubuntu/DEV/daytheipc-com/public/webeditor.html").unwrap().as_slice()
        ).unwrap().to_string();
    match user {
        Some(u)=>{
            
            println!("{}", u.is_admin);
            if u.is_admin {
                RawHtml(webeditor)
            }
            else {
                RawHtml("Hey... you are not an admin...".to_string())
            }
            
        }
        None =>{
            println!("Nada encontrado!");
            RawHtml(login)
        }
    }
    
    
    /*
    let mut document = html_editor::parse(
        std::str::from_utf8(std::fs::read("/home/ubuntu/DEV/daytheipc-com/public/webeditor.html").unwrap().as_slice()).unwrap()
        ).expect("Should load editor page");
 
    //html_editor::parse(include_str!("../public/webeditor.html")).expect("Should load editor page");

    document.insert_to(&Selector::from("div#original-document-content"), Node::Text("# aaa \n ## a \n - a \n ()[]".to_string()));
    */

}

//#[get("/editor/login?")]