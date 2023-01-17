use markdown_it::parser::inline::Text;
use markdown_it::{MarkdownIt, Node, NodeValue, Renderer};
use markdown_it::parser::core::CoreRule;

#[derive(Debug)]
// This is a structure that represents your custom Node in AST,
// it has one single argument - crab counter.
pub struct TableOfContents(usize);

// This defines how your custom node should be rendered.
impl NodeValue for TableOfContents {
    fn render(&self, node: &Node, fmt: &mut dyn Renderer) {
        // `node.attrs` are custom attributes added by other plugins
        // (for example, source mapping information)
        let mut attrs = node.attrs.clone();

        // add a custom class attribute
        attrs.push(("class", "ferris-counter".into()));

        fmt.cr(); // linebreak, multiples get merged
        fmt.open("footer", &attrs);
        #[allow(clippy::useless_format)] // for simplicity's sake
        fmt.text(&match self.0 {
            0 => format!("No crabs around here."),
            1 => format!("There is a crab lurking in this document."),
            _ => format!("There are {} crabs lurking in this document.", self.0),
        });
        fmt.close("footer");
        fmt.cr();
    }
}

// This is an extension for the markdown parser.
struct TableOfContentsRule;

impl CoreRule for TableOfContentsRule {
    // This is a custom function that will be invoked once per document.
    //
    // It has `root` node of the AST as an argument and may modify its
    // contents as you like.
    //
    fn run(root: &mut Node, _: &MarkdownIt) {
        let mut counter = 0;

        // walk through AST recursively and count the number of two
        // custom nodes added by other two rules
        const texto: Text = Text{content: "[[TOC]]".to_string()};
        
        root.walk(|node, _| {
            if node.is::<texto>() {
                counter += 1;
            }
        });

        // append a counter to the root as a custom node
        root.children.push(Node::new(TableOfContents(counter)))
    }
}

pub fn add(md: &mut MarkdownIt) {
    // insert this rule into parser
    md.add_rule::<TableOfContentsRule>();
}