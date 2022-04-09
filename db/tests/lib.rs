use std::any::Any;

use db::Create;
use rusqlite::Connection;

mod common;

//#[derive(Insert, Drop, Query, Create, Delete, Update)]
struct Person {
    //#[primarykey]
    pub id: u32,
    //#[indexed]
    pub name: String,
    pub password: Option<String>,
    //#[ignore]
    //some_internal: u32,
}

impl Person {
    // this is the very first impl
    //#[execute("INSERT INTO person (..,..) VALUES (?, ?)")]
    //#[query("SELECT * FROM person")]
    pub fn get_all(context: &dyn Any) -> Result<&dyn Iterator<Item = Person>, String> {
        //
        Err("Not implemented".to_string())
    }
}

impl Create<Connection> for Person {
    fn create(context: &Connection) -> Result<(), String> {
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
#[test]
fn create_trait() {
    let conn = Connection::open_in_memory().unwrap();
    Person::create(&conn).unwrap();
}

#[test]
fn insert() {
    let connection = common::create_connection_and_table().unwrap();

    let person1 = Person {
        id: 0,
        name: "Steffen".to_string(),
        password: Some("secret".to_string()),
    };
    let person2 = Person {
        id: 0,
        name: "Wu".to_string(),
        password: None,
    };
    {
        let mut insert_statement = connection
            .prepare("INSERT INTO person (name, password) VALUES(?, ?)")
            .unwrap();
        insert_statement
            .execute([person1.name, person1.password.unwrap()])
            .unwrap();
        insert_statement
            .execute([person2.name, "NULL".to_string()])
            .unwrap();

        let mut query_statement = connection
            .prepare("SELECT id, name, password FROM person")
            .unwrap();
        let mut person_iter = query_statement
            .query_map([], |row| {
                Ok(Person {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    password: row.get(2)?,
                })
            })
            .unwrap();

        let _res = person_iter.next().unwrap().unwrap();
        let _res = person_iter.next().unwrap().unwrap();
        assert!(person_iter.next().is_none());
    }
    connection.close().unwrap();
}

///////////////////////////////////////////////////////////////////////////////
pub struct SQLite;
///////////////////////////////////////////////////////////////////////////////
struct BufferedDaoIterator<'a> {
    max_number_buffered_rows: u32,
    current_offset: u32,
    current_row: u32,
    sql: String,
    buffered_iterator: Option<Box<dyn Iterator<Item = Person>>>,
    connection: &'a Connection,
}

impl<'a> BufferedDaoIterator<'a> {
    pub fn query(sql: &str, connection: &'a Connection) -> BufferedDaoIterator<'a> {
        BufferedDaoIterator {
            max_number_buffered_rows: 2,
            current_offset: 0,
            current_row: 0,
            sql: sql.to_owned(),
            buffered_iterator: None,
            connection,
        }
    }

    fn next_chunk(&mut self) {
        let mut sql = String::new();
        sql.push_str(self.sql.as_str());
        sql.push(' ');
        sql.push_str(" limit ");
        sql.push_str(&self.max_number_buffered_rows.to_string());
        sql.push_str(" offset ");
        sql.push_str(&self.current_offset.to_string());
        self.current_offset += self.max_number_buffered_rows;

        let connection = self.connection;

        let mut query_statement = connection.prepare(&sql).unwrap();
        {
            let stat = &mut query_statement;
            let buffered_iterator: Vec<Person> = stat
                .query_map([], |row| {
                    Ok(Person {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        password: row.get(2)?,
                    })
                })
                .unwrap()
                .filter(|item| item.is_ok())
                .map(|item| item.unwrap())
                .collect();
            self.buffered_iterator = Some(Box::new(buffered_iterator.into_iter()));
        }
    }
}

impl<'a> Iterator for BufferedDaoIterator<'a> {
    type Item = Person;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_row % self.max_number_buffered_rows == 0 {
            self.next_chunk();
        }

        let iter = self.buffered_iterator.as_mut().unwrap();
        let result = iter.next();
        self.current_row += 1;
        result
    }
}

#[test]
fn buffered_iterator() {
    let connection = common::create_connection_and_table().unwrap();

    let person1 = Person {
        id: 0,
        name: "Steffen".to_string(),
        password: Some("secret".to_string()),
    };
    let person2 = Person {
        id: 0,
        name: "Wu".to_string(),
        password: None,
    };
    let person3 = Person {
        id: 0,
        name: "Norbert".to_string(),
        password: None,
    };
    {
        let mut insert_statement = connection
            .prepare("INSERT INTO person (name, password) VALUES(?, ?)")
            .unwrap();
        insert_statement
            .execute([person1.name, person1.password.unwrap()])
            .unwrap();
        insert_statement
            .execute([person2.name, "NULL".to_string()])
            .unwrap();
        insert_statement
            .execute([person3.name, "NULL".to_string()])
            .unwrap();
    }
    //let iter = BufferedDaoIterator::<Person, SQLite>::query("SELECT * FROM person");
    let mut iter = BufferedDaoIterator::query("SELECT * FROM person", &connection);
    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    assert!(iter.next().is_some());
    assert!(iter.next().is_none());

    connection.close().unwrap();
}
