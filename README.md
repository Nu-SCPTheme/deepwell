## DEEPWELL
[![Travis CI Build Status](https://travis-ci.org/Nu-SCPTheme/deepwell.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/deepwell)

Wikidot-like storage and maintenance system. Tracks pages, revisions, files, and manages the database. Its schema and migrations are handled by [Diesel](https://diesel.rs/).

You will need the `diesel_cli` crate installed. For all commands you will need to set the
`DATABASE_URL` or `--database-url` corresponding to the database you wish to modify.

### Compilation
This library targets the latest stable Rust. At time of writing, that is 1.38.0

```sh
$ diesel migration run
$ cargo build --release
```

See [diesel.rs](https://diesel.rs/guides/getting-started/) for how to use the diesel cli tool.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).
