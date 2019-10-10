## DEEPWELL

The database schema and migrations system is managed by [Diesel](https://diesel.rs/).

**Building:**
```
$ cargo build --release
```

### Execution

You will need the `diesel_cli` crate installed. For all commands you will need to set the
`DATABASE_URL` or `--database-url` corresponding to the database you wish to modify.

**Initialization or upgrading:**
In order to set up a new database, or migrate an existing one (`diesel migration pending` can tell you if its schema is up-to-date or not), the following is used:

```
$ diesel migration run
```

**Undoing:**
```
$ diesel migration revert
```

**Creating a new migration:**
If you need to make a new database migration, a few steps are necessary. First is to generate the actual migration files themselves:

```
$ diesel migration generate [name]
```

This will create new files in `migrations/` which you edit accordingly with do/undo SQL.

After testing it works (`diesel migration redo` can be helpful with this) you run the migration locally, and commit the migration files, as well as the updated `src/schema.rs` file.

_TODO:_
Running `make gen-schema` in the project root will then produce the corresponding Typescript interfaces for the new database schema.
