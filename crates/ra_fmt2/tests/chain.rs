fn main() {
    let x = "abc".chars()
        .map(|c| c)
        .filter(|_| true)
    .collect::<String>();
}
