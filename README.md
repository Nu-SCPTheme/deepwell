## DEEPWELL
[![Travis CI Build Status](https://travis-ci.org/Nu-SCPTheme/deepwell.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/deepwell)

Wikidot-like storage and maintenance system. Tracks wikis, users, passwords, pages, revisions, files, and manages the database. Its schema and migrations are handled by [Diesel](https://diesel.rs/).

You will need the `diesel_cli` crate installed. For all commands you will need to the `DATABASE_URL` environment variable set.

### Compilation
This library targets the latest stable Rust. At time of writing, that is 1.38.0

```sh
$ diesel migration run
$ cargo build --release
```

See [diesel.rs](https://diesel.rs/guides/getting-started/) for how to use the diesel cli tool.

### Testing
```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).
