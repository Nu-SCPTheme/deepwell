## DEEPWELL

The database schema and migrations system is managed by [Diesel](https://diesel.rs/).

You will need the `diesel_cli` crate installed. For all commands you will need to set the
`DATABASE_URL` or `--database-url` corresponding to the database you wish to modify.

```sh
$ diesel migration run
$ cargo build --release
```

See [diesel.rs](https://diesel.rs/guides/getting-started/) for how to use the diesel cli tool.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).
