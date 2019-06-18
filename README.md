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
