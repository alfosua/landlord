pub mod parsers;
pub mod syntax_tree;

pub fn parse(input: &str) -> Result<syntax_tree::SyntaxTree, String> {
    match parsers::syntax_tree(input) {
        Ok((_, result)) => Ok(result),
        Err(_) => Err(String::from("Could not parse correctly.")),
    }
}
