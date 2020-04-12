use anyhow;
use kvs::Kvs;
use serde::{ Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: String,
    name: String,
}

fn main() -> anyhow::Result<()> {
    let mut s = Kvs::new(".")?;
    s.store("ymgyt", String::from("Hello ymgyt!"))?;
    s.store("ymgyt", String::from("Hello ymgyt!!"))?;
    match s.get::<String>("ymgyt") {
        Ok(Some(msg)) => println!("I got {}", msg),
        Ok(None) => println!("not found..."),
        Err(err) => println!("err: {}", err),
    }

    s.store(
        "users.AAA",
        User {
            id: "AAA".to_string(),
            name: "ymgyt".to_string(),
        },
    )?;

    match s.get::<User>("users.AAA") {
        Ok(Some(msg)) => println!("I got {:?}", msg),
        Ok(None) => println!("not found..."),
        Err(err) => println!("err: {}", err),
    }


    Ok(())
}
