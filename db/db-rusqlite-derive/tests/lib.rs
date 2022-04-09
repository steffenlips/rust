use db::{self, Create};
use db_rusqlite_derive::Create;
use rusqlite::Connection;

#[derive(Create)]
struct Person {
    #[primarykey]
    pub id: u32,
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
 */

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
    // TODO drop
}
