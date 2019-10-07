table! {
    authors (page_id, user_id, author_type) {
        page_id -> Int8,
        user_id -> Int8,
        author_type -> Text,
        created_at -> Date,
    }
}

table! {
    files (file_id) {
        file_id -> Int8,
        file_name -> Text,
        file_uri -> Text,
        description -> Text,
        page_id -> Int8,
    }
}

table! {
    pages (page_id) {
        page_id -> Int8,
        created_at -> Timestamp,
        slug -> Text,
        title -> Text,
        alt_title -> Nullable<Text>,
        tags -> Array<Text>,
    }
}

table! {
    parents (page_id, parent_page_id) {
        page_id -> Int8,
        parent_page_id -> Int8,
    }
}

table! {
    ratings (page_id, user_id) {
        page_id -> Int8,
        user_id -> Int8,
        rating -> Int2,
    }
}

table! {
    ratings_history (rating_id) {
        rating_id -> Int8,
        page_id -> Int8,
        user_id -> Int8,
        created_at -> Timestamp,
        rating -> Nullable<Int2>,
    }
}

table! {
    revisions (revision_id) {
        revision_id -> Int8,
        created_at -> Timestamp,
        page_id -> Int8,
        user_id -> Int8,
        git_commit -> Text,
        changes -> Jsonb,
    }
}

table! {
    users (user_id) {
        user_id -> Int8,
        name -> Text,
        created_at -> Timestamp,
    }
}

joinable!(authors -> pages (page_id));
joinable!(authors -> users (user_id));
joinable!(files -> pages (page_id));
joinable!(ratings_history -> pages (page_id));
joinable!(ratings_history -> users (user_id));
joinable!(revisions -> pages (page_id));
joinable!(revisions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    authors,
    files,
    pages,
    parents,
    ratings,
    ratings_history,
    revisions,
    users,
);
