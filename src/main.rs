use aruna_ulid::ArunaUlid;

fn main() {
    let ulid = ArunaUlid::generate();
    println!("{}", ulid.to_string());
}
