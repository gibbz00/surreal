use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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

    // enum_example(&db).await?;
    // insert_vec(&db).await?;
    insert_generic_struct(&db).await?;

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

async fn insert_vec(db: &Surreal<Client>) -> Result<()> {
    #[derive(Debug, Default, Serialize, Deserialize)]
    struct ParentStruct {
        data: Cow<'static, str>,
    }

    let parent_structs = vec![ParentStruct::default(), ParentStruct::default()];
    for parent_struct in parent_structs {
        db.query("CREATE parent_struct SET data = $data")
            .bind(parent_struct)
            .await?;
    }

    let res: Vec<ParentStruct> = db.select("parent_struct").await?;
    println!("{:#?}", res);
    db.query("REMOVE TABLE parent_struct").await?;

    Ok(())
}

async fn insert_generic_struct(db: &Surreal<Client>) -> Result<()> {
    #[derive(Debug, Serialize, Deserialize)]
    struct ParentStruct<T> {
        data: T,
    }

    db.query("CREATE parent_struct SET data = $data")
        .bind(ParentStruct::<Cow<'static, str>> {
            data: "test".into(),
        })
        .await?;

    let res: Vec<ParentStruct<Cow<'static, str>>> = db.select("parent_struct").await?;
    println!("{:#?}", res);
    db.query("REMOVE TABLE parent_struct").await?;

    Ok(())
}
