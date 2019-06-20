table! {
    api_tokens (id) {
        id -> Int4,
        creation_date -> Timestamp,
        value -> Text,
        scopes -> Text,
        app_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    apps (id) {
        id -> Int4,
        name -> Text,
        client_id -> Text,
        client_secret -> Text,
        redirect_uri -> Nullable<Text>,
        website -> Nullable<Text>,
        creation_date -> Timestamp,
    }
}

table! {
    comments (id) {
        id -> Int4,
        content -> Text,
        in_response_to_id -> Nullable<Int4>,
        post_id -> Int4,
        author_id -> Int4,
        creation_date -> Timestamp,
        ap_id -> Varchar,
        sensitive -> Bool,
        spoiler_text -> Text,
        public_visibility -> Bool,
    }
}

table! {
    comment_seers (id) {
        id -> Int4,
        comment_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    follows (id) {
        id -> Int4,
        follower_id -> Int4,
        following_id -> Int4,
        ap_id -> Text,
    }
}

table! {
    instances (id) {
        id -> Int4,
        public_domain -> Varchar,
        name -> Varchar,
        local -> Bool,
        blocked -> Bool,
        creation_date -> Timestamp,
        open_registrations -> Bool,
        short_description -> Text,
        long_description -> Text,
        default_license -> Text,
        long_description_html -> Varchar,
        short_description_html -> Varchar,
    }
}

table! {
    likes (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Int4,
        creation_date -> Timestamp,
        ap_id -> Varchar,
    }
}

table! {
    mentions (id) {
        id -> Int4,
        mentioned_id -> Int4,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
    }
}

table! {
    notifications (id) {
        id -> Int4,
        user_id -> Int4,
        creation_date -> Timestamp,
        kind -> Varchar,
        object_id -> Int4,
        read -> Bool,
    }
}

table! {
    password_reset_requests (id) {
        id -> Int4,
        email -> Varchar,
        token -> Varchar,
        expiration_date -> Timestamp,
    }
}

table! {
    posts (id) {
        id -> Int4,
        url -> Varchar,
        author_id -> Int4,
        title -> Varchar,
        content -> Text,
        license -> Varchar,
        creation_date -> Timestamp,
        ap_id -> Text,
        subtitle -> Text,
        slug -> Varchar,
    }
}

table! {
    reshares (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Int4,
        ap_id -> Varchar,
        creation_date -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        display_name -> Varchar,
        outbox_url -> Varchar,
        inbox_url -> Varchar,
        is_admin -> Bool,
        summary -> Text,
        email -> Nullable<Text>,
        hashed_password -> Nullable<Text>,
        instance_id -> Int4,
        creation_date -> Timestamp,
        ap_id -> Text,
        private_key -> Nullable<Text>,
        public_key -> Text,
        shared_inbox_url -> Nullable<Varchar>,
        followers_endpoint -> Varchar,
        avatar_url -> Nullable<Varchar>,
        last_fetched_date -> Timestamp,
        fqn -> Text,
        summary_html -> Text,
    }
}

joinable!(api_tokens -> apps (app_id));
joinable!(api_tokens -> users (user_id));
joinable!(comment_seers -> comments (comment_id));
joinable!(comment_seers -> users (user_id));
joinable!(comments -> posts (post_id));
joinable!(comments -> users (author_id));
joinable!(likes -> comments (comment_id));
joinable!(likes -> users (user_id));
joinable!(mentions -> comments (comment_id));
joinable!(mentions -> posts (post_id));
joinable!(mentions -> users (mentioned_id));
joinable!(notifications -> users (user_id));
joinable!(posts -> users (author_id));
joinable!(reshares -> comments (comment_id));
joinable!(reshares -> users (user_id));
joinable!(users -> instances (instance_id));

allow_tables_to_appear_in_same_query!(
    api_tokens,
    apps,
    comments,
    comment_seers,
    follows,
    instances,
    likes,
    mentions,
    notifications,
    password_reset_requests,
    posts,
    reshares,
    users,
);
