#[derive(Debug, Eq, PartialEq)]
pub enum FuzzyError {
    InvalidPoints,
    InvalidCategory(String),
    InvalidTerm(String),
    Misc(String)
}

pub type FuzzyResult<T> = Result<T, FuzzyError>;

pub type Category = String;
pub type Term = String;
pub type FuzzyValue = (Category, Term, f64);
pub type FuzzyIdent = (Category, Term);

#[macro_export]
macro_rules! values {
    ($($key:expr=>$value:expr);* $(;)*) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert(String::from($key), $value);)*
        map
    }}
}
