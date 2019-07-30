# Squs

A service to bring federated comments to static websites.

Based on Plume (but a lot of the original source have been removed or adapted).

When you build your static website, you have to run a command to tell Squs a new article was published.
It will then look for your Atom feed and federate it too the rest of the Fediverse. If someone comments it
from Mastodon/Pleroma/Plume/whatever, Squs will save these comments, and allow you to display them on your blog
with a little JavaScript snippet.

(only French speaking people with a very good sense of humor may understand the name)

## CSS classes for your comments

The Squs script does not style your comments, to let you write your own CSS and to integrate them the best with the rest of your blog.

Here are the CSS classes that you can use to style the comments:

- `error`: an error message
- `comments`: the whole comment section
- `info`: an information message
- `comment`: a single comment
- `date`: the date of publication of a comment
- `replies`: the replies to a comment
- `author`: the author's name and @

And if you want to style avatars, you can use the `.comment > header > img` selector.

## Install your instance

First of all, make sure you have a dedicated domain or subdomain for your instance.

You'll need to install these packages first :

- PosgtreSQL **or** SQLite (development files)
- Gettext
- Git
- cURL
- OpenSSL

On Debian or Ubuntu you can use one of these commands:

```bash
# For postgres
apt install postgresql postgresql-contrib libpq-dev gettext git curl gcc make openssl libssl-dev pkg-config
# For sqlite
apt install libsqlite3-dev gettext git curl gcc make openssl libssl-dev pkg-config
```

Then you need to install Rust (don't worry this script is safe):

```bash
curl https://sh.rustup.rs -sSf | sh
```

When asked, choose the *"1) Proceed with installation (default)"* option.

Then run this command to be able to run cargo in the current session:

```bash
export PATH="$PATH:/home/$USER/.cargo/bin:/home/$USER/.local/bin:/usr/local/sbin"
```

To get and compile Squs source code, use:

```bash
git clone https://github.com/BaptisteGelez/squs.git
cd squs
```

Then, you'll need to install Squs and the CLI tools to manage your instance.
Run the following commands.

```bash
# Build the back-end, replacing DATABASE either with
# postgres or sqlite depending on what you want to use
cargo install --no-default-features --features DATABASE

# Build plm, the CLI helper, replacing DATABASE again
cargo install --no-default-features --features DATABASE --path squs-cli
```

If you are using PostgreSQL, you have to create a database for Squs.

```
service postgresql start
su - postgres
createuser -d -P squs
createdb -O squs squs
```

Before starting Squs, you'll need to create a configuration file, called `.env`.
Here is a sample of what you should put inside.

```bash
# The address of the database
# (replace USER, PASSWORD, PORT and DATABASE_NAME with your values)
#
# If you are using SQlite, use the path of the database file (`squs.db` for instance)
DATABASE_URL=postgres://USER:PASSWORD@IP:PORT/DATABASE_NAME

# For PostgreSQL: migrations/postgres
# For SQlite: migrations/sqlite
MIGRATION_DIRECTORY=migrations/postgres

# The domain on which your instance will be available
BASE_URL=sq.us

# Secret key used for private cookies and CSRF protection
# You can generate one with `openssl rand -base64 32`
ROCKET_SECRET_KEY=

# Mail setting (only needed if you open registrations
# and want the password-reset feature to work (as an admin you have
# a CLI tool to reset your password))
# For more details see the Plume documentation (the variables are the same):
# https://docs.joinplu.me/environment
MAIL_SERVER=smtp.example.org
MAIL_USER=example
MAIL_PASSWORD=123456
MAIL_HELO_NAME=example.org
MAIL_ADDRESS=from@example.org
```

Now we need to run migrations. Migrations are scripts used to update
the database. To run the migrations, you can do:

```bash
squs-cli migration run
```

Migrations should be run after each update. When in doubt, run them.

After that, you'll need to setup your instance, and the admin's account.

```
squs-cli instance new
squs users new --admin
```

---

If you want to manage your Squs instance with systemd, you can use the following
unit file (to be saved in `/etc/systemd/system/squs.service`):

```ini
[Unit]
Description=squs

[Service]
Type=simple
User=YOUR USER HERE
WorkingDirectory=/home/YOUR USER HERE/squs
ExecStart=/home/YOUR USER HERE/.cargo/bin/squs
TimeoutSec=30
Restart=always

[Install]
WantedBy=multi-user.target
```

Now you need to enable this service:

```bash
systemctl enable /etc/systemd/system/squs.service
```

Now start the service:

```bash
systemctl start squs
```

Check that it is properly running:

```bash
systemctl status squs
```

Finally, you'll have to configure your reverse-proxy to bind your domain to `localhost:7878`. Here is a Caddyfile:

```
DOMAIN_NAME {
    proxy / localhost:7878 {
        transparent
    }

    header / Access-Control-Allow-Origin "*"
}
```

(where `DOMAIN_NAME` is your actual domain name)

---

And you are normally done! :tada: All you need to do is to create an account on your instance,
and follow the instructions.

---

Kudos to [Doshirae](https://home.doshi.re/) for testing Squs and helping with the documentation.
