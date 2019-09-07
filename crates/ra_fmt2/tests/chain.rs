fn main() {
    let _x = "abc".chars()
        .map(|c| c)
        .filter(|_| true)
    .collect::<String>();
}
