## DEEPWELL
[![Travis CI Build Status](https://travis-ci.org/Nu-SCPTheme/deepwell.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/deepwell)

Wikidot-like storage and maintenance system. Tracks wikis, users, passwords, pages, files, and manages the database. Requires Postgres 11 or later. Its schema and migrations are handled by [Diesel](https://diesel.rs/).

DEEPWELL provides logical, CRUD-like operations for handling low-level abstracted data such as individual page revisions.
Verification and permissions checking needs to be handled by the consumer.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have `unsafe` internals.

Currently, the following services are provided:

* **Page creation and modification:** the primary feature is the ability to make pages and set their content.
* **Wikis and multitenancy:** all pages are created in a separate, namespaced "wiki", which has its own domain and separate pages from other wikis in the same database.
* **Revisions and page history:** Tracks all changes made to a page and provides views like diffs and blames. This is implemented using a git repository, with database entries pointing to commits storing revision information.
* **Tracking authorship:** a page may have an arbitrary number of writers, translators, etc. See also the current [attribution metadata workaround](http://www.scp-wiki.net/attribution-metadata).
* **Page locks:** allows setting and clearing page locks, which the consumer can require for editing pages.
* **Rating:** tracks votes cast by users, including the ability to "neutral-vote" (vote with a value of `0`). Currently uses a `ups - downs` algorithm for scoring like Wikidot.
* **Password management:** hashes passwords with `scrypt` with configurable iteration times, as well as a delay on failed attempts to limit bruteforcing. Also blocks passwords in a specified blacklist to prevent users from using overused or weak passwords (e.g. `letmein`, `password`, etc.)
* **Login attempts:** provides a method to validate the user's password and log the attempt.

You will need the `diesel_cli` crate installed. For all commands you will need to the `DATABASE_URL` environment variable set.

### Compilation
This library targets the latest stable Rust. At time of writing, that is 1.43.0

The environment variable `DATABASE_URL` must be set in order to use diesel.
Additionally the variable `DATABASE_TEST_URL` must be set if you want to run tests.

```sh
$ diesel migration run
$ diesel migration run --database-url="$DATABASE_TEST_URL"
$ cargo build --release
```

See [diesel.rs](https://diesel.rs/guides/getting-started/) for how to use the diesel cli tool.

### Testing
```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).
