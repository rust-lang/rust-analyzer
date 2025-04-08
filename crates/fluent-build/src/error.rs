use std::io;

pub type FluentParserError =
    (fluent_syntax::ast::Resource<String>, Vec<fluent_syntax::parser::ParserError>);

#[derive(Debug)]
pub enum FluentBuildError {
    Io(io::Error),
    Parser(FluentParserError),
    /// Panic errors
    Custom(String),
}

impl From<io::Error> for FluentBuildError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<FluentParserError> for FluentBuildError {
    fn from(value: FluentParserError) -> Self {
        Self::Parser(value)
    }
}
