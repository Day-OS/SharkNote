use html_editor::{Node};

fn get_inner_text(html: &mut Vec<html_editor::Node>) -> String{
    let mut string: String;
    string = "?".to_string();
    for node in html.iter_mut() {
        match node 
            {
                Node::Text(text) =>{
                    string = text.clone();
                }
                _ =>{}
            } 
        }
    string
}
    


pub fn add_table(html: &mut Vec<html_editor::Node>) {
    let mut toc: Vec<Node> = vec![];// = vec![Node::new_element("div", vec![("id", "toc")], vec![])];
    for node in html.iter_mut() {
        match node {
            Node::Element {name, attrs, children} =>{
                if name == "h1" {
                    let h1_name = get_inner_text(children);
                    children.push(Node::new_element("a", vec![("href", "&a.1")], vec![Node::Text(h1_name.clone())])); 
                    toc.push(Node::new_element("a", vec![("href", "&a.1")], vec![Node::Text(h1_name)]));
                }
            } _ =>{}
        }
    }
}