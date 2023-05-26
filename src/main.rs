use serde::{Deserialize, Serialize};
use std::{borrow::Cow, ops::Deref};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    sql::Uuid,
    Result, Surreal,
};
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    let db = Surreal::new::<Ws>("localhost:8000").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;
    db.use_ns("unigis").use_db("testing").await?;

    enum_example(&db).await?;
    insert_generic_struct(&db).await?;
    insert_vec(&db).await?;

    Ok(())
}

async fn enum_example(db: &Surreal<Client>) -> Result<()> {
    #[derive(Debug, Serialize, Deserialize)]
    struct Address {
        designator: Designator,
    }

    #[derive(Debug, Serialize, Deserialize)]
    enum Designator {
        Street { number: usize, stair_case: String },
        Village { number: usize },
    }

    db.query("CREATE address SET designator = $designator")
        .bind(Address {
            designator: Designator::Street {
                number: 10,
                stair_case: "UV".to_string(),
            },
        })
        .await?;

    db.query("CREATE address SET designator = $designator")
        .bind(Address {
            designator: Designator::Village { number: 15 },
        })
        .await?;

    let res: Vec<Address> = db.select("address").await?;
    println!("{:#?}", res);
    db.query("REMOVE TABLE address").await?;

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

#[tokio::test]
async fn test1() {
    let db = DecoupledDatabase::new().await;
    db.query("SELECT * FROM count(1)").await.unwrap();
    db.drop().await;
}

#[tokio::test]
async fn test2() {
    let db = DecoupledDatabase::new().await;
    db.query("SELECT * FROM count(1)").await.unwrap();
    db.drop().await;
}

struct DecoupledDatabase {
    name: Uuid,
    db: Surreal<Client>,
}

impl DecoupledDatabase {
    pub async fn new() -> Self {
        let name = Uuid::new_v4();
        let db = Surreal::new::<Ws>("localhost:8000").await.unwrap();
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .unwrap();
        db.use_ns(name.simple().to_string())
            .use_db("testing")
            .await
            .unwrap();
        DecoupledDatabase { name, db }
    }

    // Workaround since async drop hasn't been implemented.
    pub async fn drop(self) {
        let sql = format!("REMOVE NAMESPACE {};", self.name.simple());
        self.query(sql).await.unwrap();
    }
}

impl Deref for DecoupledDatabase {
    type Target = Surreal<Client>;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
