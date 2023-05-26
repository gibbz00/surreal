use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Result, Surreal,
};

#[tokio::main]
async fn main() -> Result<()> {
    let db = Surreal::new::<Ws>("localhost:8000").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;
    db.use_ns("namespace").use_db("database").await?;

    enum_example(&db).await?;

    Ok(())
}

async fn enum_example(db: &Surreal<Client>) -> Result<()> {
    #[derive(Debug, Serialize, Deserialize)]
    enum Address {
        Street(usize),
        Metric(usize),
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ParentStruct {
        address: Address,
    }

    db.query("CREATE parent_struct SET address = $address")
        .bind(ParentStruct {
            address: Address::Street(10),
        })
        .await?;
    db.query("CREATE parent_struct SET address = $address")
        .bind(ParentStruct {
            address: Address::Metric(11),
        })
        .await?;

    let res: Vec<ParentStruct> = db.select("parent_struct").await?;
    println!("{:#?}", res);
    for st in res {
        match st.address {
            Address::Street(_) => println!("Found street address!"),
            Address::Metric(_) => println!("Found metric address!"),
        }
    }

    db.query("REMOVE TABLE parent_struct").await?;
    Ok(())
}
