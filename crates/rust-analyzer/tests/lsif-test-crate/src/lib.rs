#![allow(unused)]

macro_rules! generate_const_from_identifier(
    ($id:ident) => (
        const _: () = { const $id: &str = "encoded_data"; };
    )
);

generate_const_from_identifier!(REQ_001);
mod tests {
    use super::*;
    generate_const_from_identifier!(REQ_002);
}
