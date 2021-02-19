# dclims

Command line tool for returning item information from the Destiny 2 manifest from hash ids (from API calls).

May return more than one result.


## USAGE
```
USAGE:
    dclims [FLAGS] [OPTIONS] --hash <hash>

FLAGS:
        --help       
            Prints help information

    -V, --version    
            Prints version information

    -v, --verbose    
            Print out additional information
            
            Output is printed to stderr.

OPTIONS:
    -D, --data-dir <data-dir>       
            Directory where Destiny 2 manifest database file is stored. (optional)
            
            This will normally be downloaded using the dclim tool, and stored in a file named manifest.sqlite3 (in the
            manifest directory specified when running dclim).
    -h, --hash <hash>               
            The hash id from the Destiny 2 API for the item to be searched for.
            
            Example : 326060471
    -O, --output-format <output>    
            Format for command output
            
            Valid values are default (Default) and tsv.
            
            tsv outputs in a tab (\t) seperated format of columns with lines ending in a new line character (\n).
            [default: default]
```

Manifest can be downloaded and synced with from [dclim](https://github.com/mikechambers/dcli/tree/main/src/dclim).

### Examples

#### Retrieve information for *Luna's Howl* by its API hash id.
```
$ dclims -hash 3260604718
```

which returns:

```
Found 1 item
-----------------------------
Name           Luna's Howl
Description    "Guardians never die. But we don't forget those who do." —Lord Shaxx
Has Icon       true
Icon Path      https://www.bungie.net/common/destiny2_content/icons/f59ce6481de388222f6ed740ed829fb1.jpg
```

#### Retrieve information for *Luna's Howl* by its API hash id and output to tab seperated format (tsv)

```
$ dclims --hash 153979396 --output-format tsv
```

outputs:

```
0       Luna's Howl     "Guardians never die. But we don't forget those who do." —Lord Shaxx    true    https://www.bungie.net/common/destiny2_content/icons/f59ce6481de388222f6ed740ed829fb1.jpg
```
## Questions, Feature Requests, Feedback

If you have any questions, feature requests, need help, are running into issues, or just want to chat, join the [dcli Discord server](https://discord.gg/2Y8bV2Mq3p).

You can also log bugs and features requests on the [issues page](https://github.com/mikechambers/dcli/issues).

## Compiling

This utility is written and compiled in [Rust](https://www.rust-lang.org/).

When compiling you must have an environment variable named `DESTINY_API_KEY` which contains your [Bungie API key](https://www.bungie.net/en/Application).

To compile, switch to the `src/` directory and run:

```
$ cargo build --release
```

which will place the compiled tools in *src/target/release*
