use std::io::Read;

fn read_stdin() -> String {
    let mut buff = String::new();
    std::io::stdin().read_to_string(&mut buff).expect("Cannot read from stdin!");

    buff
}

fn main() {
    let input = read_stdin();
    let task = serde_json::from_str(&input).expect(&format!("Cannot parse '{}'", &input));
    let results = ra_proc_macro_srv::expand_task(&task);

    println!("{}", &serde_json::to_string(&results).expect("Cannot serialize results!"));
}
