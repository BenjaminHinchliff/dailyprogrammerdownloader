# r/dailyprogrammer Downloader

a simple (to use) tool to download posts from the subreddit r/dailyprogrammer.

## Usage

You can find releases on the releases page.

see the help screen:

```
Daily Programmer Querier 0.1.0
Benjamin Hinchliff <benjamin.hinchliff@gmail.com>
a cli interface for r/dailyprogrammer built with the reddit api, rust, and a lot of parsing code

USAGE:
    dailyprogrammerquerier.exe [FLAGS] <id> <difficulties>...

FLAGS:
    -h, --help       Prints help information
    -r, --refresh    refreshes the list of r/dailyprogrammer posts and saves it to the cache
    -V, --version    Prints version information

ARGS:
    <id>                 the id of the challenge to get
    <difficulties>...    the difficult(ies) of the id to get
```

### A Note About The Tool

Unfortunately, this tool fails to download the more recent posts. This is because I'm using the [r/dailyprogrammer challenges wiki page](https://www.reddit.com/r/dailyprogrammer/wiki/challenges) for a list of challenges. It seems that this page is really out of date (the latest challenges are from 2017!) so it's not the best source. The reason I didn't just have it search for the challenges is that the names of the posts don't have a consistent format, not to mention that reddit's search api can be inconsistent with what it returns (because it's designed for users not cli tools). If anyone has an idea where I can find a complete list of challenges I'd greatly appreciate you telling me.
