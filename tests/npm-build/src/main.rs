const GENERATED: &str = include_str!(concat!(env!("OUT_DIR"), "/generated.in"));

fn main() {
    print!("{}", GENERATED);
}
