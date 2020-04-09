use kvs::Kvs;
fn main() {
    if let Err(e) = Kvs::new(".") {
        println!("{}", e);
    }
    println!("run kvs!");
}
