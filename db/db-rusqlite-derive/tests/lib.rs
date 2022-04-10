use db::{self, Create, Drop, Insert};
use db_rusqlite_derive::{Create, Drop};
use rusqlite::Connection;

#[derive(Create, Drop)]
struct Person {
    #[primarykey]
    pub id: usize,
    //#[indexed]
    pub name: String,
    pub password: Option<String>,
    //#[ignore]
    //some_internal: u32,
}
/*impl db::Create<rusqlite::Connection> for Person {
    fn create(context: &rusqlite::Connection) -> Result<(), String> {
        context
            .execute(
                "CREATE TABLE person (
                            id    INTEGER PRIMARY KEY,
                            name  TEXT NOT NULL,
                            password  TEXT
                        )",
                [],
            )
            .unwrap();
        Ok(())
    }
}

impl db::Drop<rusqlite::Connection> for Person {
    fn drop(context: &rusqlite::Connection) -> Result<(), String> {
        context.execute("DROP TABLE person", []).unwrap();
        Ok(())
    }
}
*/
impl db::Insert<rusqlite::Connection> for Person {
    fn insert(&mut self, context: &rusqlite::Connection) -> Result<(), String> {
        let mut insert_statement = context
            .prepare("INSERT INTO person (name, password) VALUES(?, ?)")
            .or_else(|err| Err(format!("Error creating executing sql ({})", err)))?;

        let param_name = &self.name;
        let param_password = match &self.password {
            Some(password) => password,
            None => "NULL",
        };

        self.id = insert_statement
            .execute([param_name, param_password])
            .or_else(|err| Err(format!("Error while executing sql ({})", err)))?;

        Ok(())
    }
}
#[test]
fn create_and_drop_table() {
    let conn = Connection::open_in_memory().unwrap();

    let mut test_statement = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='person'")
        .unwrap();
    {
        let mut test = test_statement.query([]).unwrap();
        assert!(test.next().unwrap().is_none());
    }
    Person::create(&conn).unwrap();
    {
        let mut test = test_statement.query([]).unwrap();
        assert!(test.next().unwrap().is_some());
    }
    Person::drop(&conn).unwrap();
    {
        let mut test = test_statement.query([]).unwrap();
        assert!(test.next().unwrap().is_none());
    }
}
#[test]
fn insert_all_values() {
    let conn = Connection::open_in_memory().unwrap();
    Person::create(&conn).unwrap();

    let mut person = Person {
        id: 0,
        name: "Paul".to_owned(),
        password: Some("Pass".to_owned()),
    };
    let result = person.insert(&conn);
    assert!(result.is_ok());
    assert_ne!(person.id, 0);
}
#[test]
fn insert_only_mandatory_values() {
    let conn = Connection::open_in_memory().unwrap();
    Person::create(&conn).unwrap();

    let mut person = Person {
        id: 0,
        name: "Paul".to_owned(),
        password: None,
    };
    let result = person.insert(&conn);
    assert!(result.is_ok());
    assert_ne!(person.id, 0);
}
