-- Your SQL goes here
CREATE TABLE instances (
    id SERIAL PRIMARY KEY,
    public_domain VARCHAR NOT NULL UNIQUE,
    name VARCHAR NOT NULL,
    local BOOLEAN NOT NULL DEFAULT 'f',
    blocked BOOLEAN NOT NULL DEFAULT 'f',
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    open_registrations BOOLEAN NOT NULL DEFAULT 't',
	short_description TEXT NOT NULL DEFAULT '',
	long_description TEXT NOT NULL DEFAULT '',
	default_license TEXT NOT NULL DEFAULT 'CC-BY-SA',
	-- Your SQL goes here
	long_description_html VARCHAR NOT NULL DEFAULT '',
	short_description_html VARCHAR NOT NULL DEFAULT ''
);

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR NOT NULL,
    display_name VARCHAR NOT NULL DEFAULT '',
    outbox_url VARCHAR NOT NULL UNIQUE,
    inbox_url VARCHAR NOT NULL UNIQUE,
    is_admin BOOLEAN NOT NULL DEFAULT 'f',
    summary TEXT NOT NULL DEFAULT '',
    email TEXT,
    hashed_password TEXT,
    instance_id INTEGER REFERENCES instances(id) ON DELETE CASCADE NOT NULL,
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    ap_id TEXT NOT NULL DEFAULT '' UNIQUE,
	private_key TEXT,
	public_key TEXT NOT NULL DEFAULT '',
	shared_inbox_url VARCHAR,
	followers_endpoint VARCHAR NOT NULL DEFAULT '' UNIQUE,
    avatar_url VARCHAR,
	last_fetched_date TIMESTAMP NOT NULL DEFAULT now(),
	fqn TEXT NOT NULL DEFAULT '' UNIQUE,
	summary_html TEXT NOT NULL DEFAULT '',
	CONSTRAINT users_unique UNIQUE (username, instance_id)
);

CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    url VARCHAR NOT NULL UNIQUE,
    author_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    license VARCHAR NOT NULL DEFAULT 'CC-0',
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    ap_id TEXT NOT NULL DEFAULT '' UNIQUE,
    subtitle TEXT NOT NULL DEFAULT ''
);

CREATE TABLE follows (
    id SERIAL PRIMARY KEY,
    follower_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    following_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    ap_id TEXT NOT NULL DEFAULT '' UNIQUE
);

CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL DEFAULT '',
    in_response_to_id INTEGER REFERENCES comments(id),
    post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE NOT NULL,
    author_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    ap_id VARCHAR NOT NULL UNIQUE,
    sensitive BOOLEAN NOT NULL DEFAULT 'f',
    spoiler_text TEXT NOT NULL DEFAULT '',
    public_visibility BOOLEAN NOT NULL DEFAULT 't'
);

CREATE TABLE likes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    comment_id INTEGER REFERENCES comments(id) ON DELETE CASCADE NOT NULL,
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    ap_id VARCHAR NOT NULL DEFAULT '' UNIQUE,
    CONSTRAINT likes_unique UNIQUE (user_id, comment_id)
);

CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
	kind VARCHAR NOT NULL DEFAULT 'unknown',
	object_id INTEGER NOT NULL DEFAULT 0,
    read BOOLEAN NOT NULL DEFAULT 'f'
);

CREATE TABLE reshares (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    comment_id INTEGER REFERENCES comments(id) ON DELETE CASCADE NOT NULL,
    ap_id VARCHAR NOT NULL DEFAULT '' UNIQUE,
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    CONSTRAINT reshares_unique UNIQUE (user_id, comment_id)
);

CREATE TABLE mentions (
    id SERIAL PRIMARY KEY,
    mentioned_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    post_id INTEGER REFERENCES posts(id) ON DELETE CASCADE,
    comment_id INTEGER REFERENCES comments(id) ON DELETE CASCADE
);

CREATE TABLE apps (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL DEFAULT '',
    client_id TEXT NOT NULL,
    client_secret TEXT NOT NULL,
    redirect_uri TEXT,
    website TEXT,
    creation_date TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE api_tokens (
    id SERIAL PRIMARY KEY,
    creation_date TIMESTAMP NOT NULL DEFAULT now(),
    value TEXT NOT NULL,
    scopes TEXT NOT NULL,
    app_id INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT api_tokens_unique_value UNIQUE (value)
);

CREATE TABLE comment_seers (
    id SERIAL PRIMARY KEY,
    comment_id INTEGER REFERENCES comments(id) ON DELETE CASCADE NOT NULL,
    user_id INTEGER REFERENCES users(id) ON DELETE CASCADE NOT NULL,
    UNIQUE (comment_id, user_id)
);

CREATE TABLE password_reset_requests (
  id SERIAL PRIMARY KEY,
  email VARCHAR NOT NULL,
  token VARCHAR NOT NULL,
  expiration_date TIMESTAMP NOT NULL
);

CREATE INDEX password_reset_requests_token ON password_reset_requests (token);
CREATE UNIQUE INDEX password_reset_requests_email ON password_reset_requests (email);
