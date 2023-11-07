// @generated automatically by Diesel CLI.

diesel::table! {
    idioms (id) {
        id -> Integer,
        phrase -> Text,
        example -> Nullable<Text>,
        created_at -> Timestamp,
    }
}
