///////////////////////////////////////////////////////////////////////////////
/// Trait Create
/// creates the SQL string for table creation
/// If the table has indexed columns it creates the index sql, too
pub trait Create<DbContext> {
    fn create(context: &DbContext) -> Result<(), String>;
}
///////////////////////////////////////////////////////////////////////////////
/// Trait Drop
pub trait Drop<DbContext> {
    fn drop(context: &DbContext) -> Result<(), String>;
}
///////////////////////////////////////////////////////////////////////////////
/// Trait InsertTable
pub trait Insert<DbContext> {
    fn insert(&mut self, context: &DbContext) -> Result<(), String>;
}
/*///////////////////////////////////////////////////////////////////////////////
/// Trait InsertTable
pub trait Query<DbContext> {
    fn query_one(context: &DbContext) -> Result<(), String>;
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
 */
