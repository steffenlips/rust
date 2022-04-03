use std::{marker::PhantomData, rc::Rc};

pub trait Context {}

///////////////////////////////////////////////////////////////////////////////
/// Trait Create
/// creates the SQL string for table creation
/// If the table has indexed columns it creates the index sql, too
pub trait Create {
    fn create(context: &dyn Context) -> Result<(), String>;
}

///////////////////////////////////////////////////////////////////////////////
/// Trait Drop
pub trait Drop {
    fn drop(context: &dyn Context) -> Result<(), String>;
}

///////////////////////////////////////////////////////////////////////////////
/// Trait InsertTable
pub trait Insert<DAO> {
    fn insert(context: &dyn Context, dao: &DAO) -> Result<(), String>;
}

///////////////////////////////////////////////////////////////////////////////
/// Trait Delete
pub trait Delete<DAO> {
    fn delete_by(context: &dyn Context, dao: &DAO) -> Result<(), String>;
}

///////////////////////////////////////////////////////////////////////////////
/// Trait QueryTable
pub trait Query<'a, DAO> {
    fn query(
        context: &dyn Context,
        column: &str,
        value: &str,
    ) -> Result<&'a dyn Iterator<Item = DAO>, String>;
}

///////////////////////////////////////////////////////////////////////////////
/// Trait Update
pub trait Update<DAO> {
    fn update(context: &dyn Context, dao: &DAO) -> Result<(), String>;
}
