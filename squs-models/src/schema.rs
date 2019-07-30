table! {
    api_tokens (id) {
        id -> Integer,
        creation_date -> Timestamp,
        value -> Text,
        scopes -> Text,
        app_id -> Integer,
        user_id -> Integer,
    }
}

table! {
    apps (id) {
        id -> Integer,
        name -> Text,
        client_id -> Text,
        client_secret -> Text,
        redirect_uri -> Nullable<Text>,
        website -> Nullable<Text>,
        creation_date -> Timestamp,
    }
}

table! {
    comment_seers (id) {
        id -> Integer,
        comment_id -> Integer,
        user_id -> Integer,
    }
}

table! {
    comments (id) {
        id -> Integer,
        content -> Text,
        in_response_to_id -> Nullable<Integer>,
        post_id -> Integer,
        author_id -> Integer,
        creation_date -> Timestamp,
        ap_id -> Text,
        sensitive -> Bool,
        spoiler_text -> Text,
        public_visibility -> Bool,
    }
}

table! {
    follows (id) {
        id -> Integer,
        follower_id -> Integer,
        following_id -> Integer,
        ap_id -> Text,
    }
}

table! {
    instances (id) {
        id -> Integer,
        public_domain -> Text,
        name -> Text,
        local -> Bool,
        blocked -> Bool,
        creation_date -> Timestamp,
        open_registrations -> Bool,
        short_description -> Text,
        long_description -> Text,
        default_license -> Text,
        long_description_html -> Text,
        short_description_html -> Text,
    }
}

table! {
    likes (id) {
        id -> Integer,
        user_id -> Integer,
        comment_id -> Integer,
        creation_date -> Timestamp,
        ap_id -> Text,
    }
}

table! {
    mentions (id) {
        id -> Integer,
        mentioned_id -> Integer,
        post_id -> Nullable<Integer>,
        comment_id -> Nullable<Integer>,
    }
}

table! {
    notifications (id) {
        id -> Integer,
        user_id -> Integer,
        creation_date -> Timestamp,
        kind -> Text,
        object_id -> Integer,
        read -> Bool,
    }
}

table! {
    password_reset_requests (id) {
        id -> Integer,
        email -> Text,
        token -> Text,
        expiration_date -> Timestamp,
    }
}

table! {
    posts (id) {
        id -> Integer,
        url -> Text,
        author_id -> Integer,
        title -> Text,
        content -> Text,
        license -> Text,
        creation_date -> Timestamp,
        ap_id -> Text,
        subtitle -> Text,
        slug -> Text,
    }
}

table! {
    reshares (id) {
        id -> Integer,
        user_id -> Integer,
        comment_id -> Integer,
        ap_id -> Text,
        creation_date -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Integer,
        username -> Text,
        display_name -> Text,
        outbox_url -> Text,
        inbox_url -> Text,
        is_admin -> Bool,
        summary -> Text,
        email -> Nullable<Text>,
        hashed_password -> Nullable<Text>,
        instance_id -> Integer,
        creation_date -> Timestamp,
        ap_id -> Text,
        private_key -> Nullable<Text>,
        public_key -> Text,
        shared_inbox_url -> Nullable<Text>,
        followers_endpoint -> Text,
        avatar_url -> Nullable<Text>,
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
    comment_seers,
    comments,
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
