# dcli

dcli (Destiny Command Line Interface) is a collection of utilities and apps that provide a command line interface (CLI) for viewing player stats and data from Destiny 2, using the [Destiny 2 API](https://github.com/Bungie-net/api).

[![](images/dcliah_x_sm.png)](images/dcliah.png)
[![](images/dcliad_x_sm.png)](images/dcliad.png)

If you run into any issues, have any ideas, or just want to chat, please post in [issues](https://github.com/mikechambers/dcli/issues) or share on [Discord](https://discord.gg/2Y8bV2Mq3p)


## Apps

### Utilities
| TOOL | DESCRIPTION |
| --- | --- |
| [dclis](https://github.com/mikechambers/dcli/tree/main/src/dclis) | Retrieves primary platform and membership ids for Destiny 2 players |
| [dclim](https://github.com/mikechambers/dcli/tree/main/src/dclim) | Manages and syncs the remote Destiny 2 API manifest database |
| [dclias](https://github.com/mikechambers/dcli/tree/main/src/dclias) | Downloads and syncs Destiny 2 Crucible activity history into a local sqlite3 database file |
| [dclic](https://github.com/mikechambers/dcli/tree/main/src/dclic) | Retrieves character ids for the specified member |
| [dclims](https://github.com/mikechambers/dcli/tree/main/src/dclims) | Searches the Destiny 2 manifest by hash ids (from API calls) |
| [dclitime](https://github.com/mikechambers/dcli/tree/main/src/dclitime) | Generates date / time stamps for Destiny 2 weekly event moments |


### Apps
| TOOL | DESCRIPTION |
| --- | --- |
| [dclia](https://github.com/mikechambers/dcli/tree/main/src/dclia) | Displays information on player's current activity within Destiny 2 |
| [dcliah](https://github.com/mikechambers/dcli/tree/main/src/dcliah) | Displays Destiny 2 activity history and stats |
| [dcliad](https://github.com/mikechambers/dcli/tree/main/src/dcliad) | Displays Destiny 2 Crucible activity / match details |


### Libraries
| TOOL | DESCRIPTION |
| --- | --- |
| [dcli](https://github.com/mikechambers/dcli/tree/main/src/dcli) | Library used across all of the dcli apps |

Each tool page contains additional tool specific information and usage examples.

You can also find some additional examples in the [examples](examples/) folder.

## Download and Installation

You can download the latest binaries for Windows, Linux and x86_64 Mac from the [releases](https://github.com/mikechambers/dcli-gha/releases/latest) page.

Just download, place them in your path and run from the command line (use --help to get a list of options). You can find a script [here](https://github.com/mikechambers/dcli/blob/main/tests/) that will run all of the apps to verfiy they are working and in your path.

[![](https://img.shields.io/github/v/release/mikechambers/dcli?style=social)](https://github.com/mikechambers/dcli-gha/releases/latest)

**IMPORTANT**: Mac binaries are not signed, which can cause some hassle the first time you run them. You can find info on how to easily run them [here](https://github.com/mikechambers/dcli/wiki/Running-dcli-tools-on-Mac-OS-X).

## Getting Started

The core idea behind the project is to provide small, focused utilities that provide useful info by themselves, but that can also be combined together, or with other shell scripts to create greater functionality.

To get started, download the release (or compile from source), and place the executables somewhere within your path so you can call them from anywhere. Doing this will make it easy to call from anywhere on your system and from other sciprts.

If you are running on Mac, make sure to [read this article](https://github.com/mikechambers/dcli/wiki/Running-dcli-tools-on-Mac-OS-X) to ensure everything will run correctly.

In general, there are 3 steps to take before you can begin getting data:

1. Get your member id and platform (dclis)
2. Download the manifest (dclim) and sync your activities (dclias)
3. View your stats (dcliah)



### Retrieve your member id, platform and character ids

The first thing we will do is retrieve your Destiny member id and platform for your primary Destiny account. 
```
$ dclis --name mesh --platform xbox
```

This will output something like:

```
Display Name   mesh
id             4611686018429783292
Platform       Xbox
Platform Id    1
```

Note that the platform may be different that what you entered depending on whether you have set up cross save.

Most of the apps that call the API will require you to pass in your member id and platform.

### (Optional) Save data in environment variables

One useful trick, is to store some of the data you need to reuse, such your member id, in environment variables.

For example, on Linux / Mac OS X, I have this placed in my `.profile` file:

```
export MEMBER_ID=4611686018429783292
export PLATFORM=xbox
```

Then, I can just use `$MEMBER_ID` whenever you need to use it.

Here are some resources going over how to set environment variables on [Mac OS X](https://apple.stackexchange.com/questions/106778/how-do-i-set-environment-variables-on-os-x), [Linux](https://www.serverlab.ca/tutorials/linux/administration-linux/how-to-set-environment-variables-in-linux/) and [Windows](https://support.shotgunsoftware.com/hc/en-us/articles/114094235653-Setting-global-environment-variables-on-Windows).


At this point, we have all of our data setup, and can access it via environment variables like so:

```
$ echo $MEMBER_ID
```

or on Windows

```
$ echo $env:MEMBER_ID
```

Storing this data in enviroment variables is not required but makes it much easier to use the apps. The examples below will assume you are using environment variables (if not you can just enter the actual data values in place of the variables).

### Download the manifest

The next thing we need to do is to download the Destiny 2 manifest database using dclim. This contains information about all of the items and activities in Destiny 2, and is updated periodically.

Just run:

```
$ dclim
```

and the manifest file will be downloaded and saved in a system appropriate directory. You should peridocially run this command to check whether the manifest has been updated by Bungie.

### Sync your activities

Next, lets sync all of our activity history to a local database. This data will be used by other apps, such as dcliah to generate and display stats.

```
$ dclias --member-id $MEMBER_ID --platform $PLATFORM
```

The first time you run this, it may take a couple of minutes to load all of your data (depending on the number of activities that you have). If any errors occur while syncing, just re-run the app when its done. It is smart enough to only sync the acitivties that it missed the first time.

Once you have done the initial sync, subsequent activity syncs should be very fast. You can periodically re-run the app to keep things in sync, or have dcliah / dcliad automatically sync before it displays your stats.

### Grabbing data

Now that we have everything set up, we can now start using the apps to view our data.

Let's start getting some data. Lets see whether we are playing Destiny 2, and if so, which activity:

```
$ dclia --member-id $MEMBER_ID --platform $PLATFORM
```

Lets see all of our Crucible stats since the weekly reset on Tuesday for our last played character:

```
$ dcliah --member-id $MEMBER_ID --platform $PLATFORM --moment weekly
```

Lets view our historic Crucible stats across all of our characters for all time:

```
$ dcliah --member-id $MEMBER_ID --platform xbox --mode all_pvp --moment all_time -class all
```

### Putting it all together

These can be useful on their own, but where they can be really powerful is when you start to customize them for how you want to use them.

There are a couple of examples in the [examples directory](https://github.com/mikechambers/dcli/tree/main/examples):

* Send a notification when you load into a new activity (particularly useful when playing crucible so you can see which map you are loading into)
* Automatically generate weekly reports on your Crucible stats and email them to yourself
* Track your Crucible stats per game play session

As you can see, right now, a lot of the functionality is Crucible based. If you would like to see other functionality, make sure you requests it in the [issues](https://github.com/mikechambers/dcli/issues), or [Discord](https://discord.gg/2Y8bV2Mq3p).

### Learning More

At anytime, you can see which arguments and options are avaliable by passing the *--help* argument:

```
$ dcliah --help
```

You can also find additional documentation and examples on the [individual app pages for each app](https://github.com/mikechambers/dcli).

## Questions, Feature Requests, Feedback

If you have any questions, feature requests, need help, or just want to chat, join the [dcli Discord server](https://discord.gg/2Y8bV2Mq3p).

You can also log bugs and features requests on the [issues page](https://github.com/mikechambers/dcli/issues).

## Compiling

Tools are written and compiled in [Rust](https://www.rust-lang.org/).

![Build Status](https://github.com/mikechambers/dcli/workflows/dcli/badge.svg)

When compiling you must have an environment variable named `DESTINY_API_KEY` which contains your [Bungie API key](https://www.bungie.net/en/Application).

To compile, switch to the `src/` directory and run:

```
$ cargo build --release
```

which will place the compiled tools in *src/target/release*

## Known Issues

* Tied matches are not displayed correctly, and are treated as a Victory.

## License

Project released under a [MIT License](LICENSE.md).

[![License: MIT](https://img.shields.io/badge/License-MIT-orange.svg)](LICENSE.md)
