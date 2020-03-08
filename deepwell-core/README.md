## DEEPWELL Core

Library with published data types from [DEEPWELL](https://github.com/Nu-SCPTheme/deepwell). See that repository for more information on what DEEPWELL is and how you might use it.

This crate publishes three categories of item:

The first is the `Error` type, which represents possible failure modes of DEEPWELL operations.

Second are the collection of `Id` types, which are cheap wrappers around an `i64`. This ensures that arithmetic cannot be performed on them and raises the confidence that it actually corresponds to a row in the database.

Then there are models, which are `diesel`-queryable representations of rows retrieved from disk. They are serializable and read-only.

Finally, this crate also contains the `Votes` model and the trait `Scorer` and a number of implementations for it. This permits choosing from a number of different means of ranking articles based on their vote composition, or extending it with a custom implementation.
