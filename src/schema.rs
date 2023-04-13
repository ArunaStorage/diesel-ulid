// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Uuid,
        body -> Text,
        published -> Bool,
    }
}
