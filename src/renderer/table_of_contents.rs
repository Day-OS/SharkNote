use html_editor::Node;
extern crate unidecode;
use unidecode::unidecode;

fn get_inner_text(html: &mut Vec<html_editor::Node>) -> String {
    let mut string: String;
    string = "?".to_string();
    for node in html.iter_mut() {
        match node {
            Node::Text(text) => {
                string = text.clone();
            }
            _ => {}
        }
    }
    string
}

pub fn add_table(html: &mut Vec<html_editor::Node>, toc: &mut Vec<html_editor::Node>) {
    let mut idnumber: u16 = 0;
    for node in html.iter_mut() {
        match node {
            Node::Element {
                name,
                attrs,
                children,
            } => {
                if name == "h1" {
                    idnumber += 1;
                    let mut h1_name: String = idnumber.to_string();
                    h1_name.push_str(". ");
                    h1_name.push_str(&get_inner_text(children));
                    let mut id: String = "".to_string();
                    id.push_str(&idnumber.to_string());
                    id.push_str(&unidecode(&h1_name.replace(" ", "").to_lowercase()).to_string());
                    let mut href_id: String = "#".to_string();
                    href_id.push_str(&id);
                    children.push(Node::new_element("a", vec![("name", &id)], vec![]));
                    toc.push(Node::new_element(
                        "a",
                        vec![("href", &href_id)],
                        vec![Node::Text(h1_name)],
                    ));
                }
            }
            _ => {}
        }
    }
}
