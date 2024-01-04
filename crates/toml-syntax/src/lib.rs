use taplo::parser;

pub use taplo::dom;

pub fn parse_toml<'a>(content: &'a str) -> parser::Parse {
    parser::parse(content)
}
