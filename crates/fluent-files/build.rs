fn main() {
    println!("cargo::rerun-if-changed=src/");
    let result = fluent_build::build_fluent_src();
    if let Err(r) = result {
        panic!("{r:#?}")
    }
}
