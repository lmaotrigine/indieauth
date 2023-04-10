// @generated automatically by Diesel CLI.

diesel::table! {
    gitlab_tokens (id) {
        id -> Text,
        user_id -> Integer,
        access_token -> Text,
        refresh_token -> Text,
    }
}

diesel::table! {
    indieauth_codes (code) {
        code -> Text,
        client_id -> Text,
        redirect_uri -> Text,
        state -> Text,
        response_type -> Text,
        code_challenge -> Text,
        authorized -> Bool,
    }
}

diesel::table! {
    tokens (id) {
        id -> Text,
        sub -> Text,
        aud -> Text,
        iss -> Text,
        iat -> Text,
        exp -> Nullable<Integer>,
        valid -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(gitlab_tokens, indieauth_codes, tokens,);
