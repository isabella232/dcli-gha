# dcli Release Notes

## v0.5.62
* Fixed bug that could cause some activities to never sync property (and could throw RowNotFound error.) Requires all data to be re-synced.
* Fixed issues where errors would occur if new data is found in API, and manifest hasnt been updated yet.
* Updated required compiler version, and a number of libraries.

## v0.5.61 February 7, 2021
* Added moment for Season of the Chosen

## v0.5.6 February 7, 2021
* Display mercy data in dcliah

## v0.5.5 January 24, 2021
* Added player rating to dcliad (this is based on Destiny combat rating, similar to elo)
* Fixed bug where wrong platform was being stored for players. Requires database update and data to be re-synced
* Added --end-moment argument to dcliah to allow specifying start / end moments to search.
* Added moments for each season (see docs for [dcliah](https://github.com/mikechambers/dcli/tree/main/src/dcliah))
* Some performance optimizations for data store queries

## v0.5.1 January 22, 2021
* Re-releasing as Windows Defender falsly flagged dcliah as containing malware (known issue with Defender). Hoping it won't flag new build.

## v0.5.0 January 22, 2021
* Released dcliad (per-game details)
* Removed dclics (last included in v0.3.71)
* removed stats_report example (required dclics)
* Updated data store format. Will require all data to be re-downloaded
* Data for all activity players is now stored in data store (previously only stored data for specified player)
* dcliah added some additional data to stat output, including --activity-index, which can be used to load game specific data in dcliad

## v0.3.71 January 14, 2021
* Fixed bug that led to wrong weapon stats when used in 1 game
* Fixed bug preventing import of data of multiple players when they had played in same activity

## v0.3.7 : January 11, 2021
* Added additional weapon metrics

## v0.3.6 : January 6, 2021
* Added support to sort weapon stat results in dcliah (--weapon-sort)

## v0.3.5 : January 6, 2021
* Added support for private matches. See [issue 10](https://github.com/mikechambers/dcli/issues/10) for current limitations
* General optimizations and performance improvements
* This update will require that all activity data be redownloaded

## v0.3.2 : January 1, 2021

* Fixed wrong version numbers in some apps
* Update Copyright year

## v0.3.1 : December 30, 2020

* Updated tests to run with latest release

## v0.3.0 : December 30, 2020

* Refactored dcliah to use local data
* Added dclias
* Deprecated dclics
* Added default storage locations for data storage. No longer need to specify manifest-dir for apps
* Simplified and standardized argument names. Please review docs

## v0.2.0 : December 14, 2020

* Added dcliah
* Added dclitime
* Updated and standardized all tool command line arguments (see docs)
* Added examples/session (bash and Windows Powershell)
* Lots of fixes and optimizations

## v0.1.1 : December 3, 2020

* Fix [#1](https://github.com/mikechambers/dcli/issues/1)
* Fix [#2](https://github.com/mikechambers/dcli/issues/2)
* Fix [#3](https://github.com/mikechambers/dcli/issues/3)
* Updated [status_notification_win.ps1](https://github.com/mikechambers/dcli/blob/main/examples/status_notification_win.ps1). Requires dclia v0.1.1

## v0.1.0 : December 2, 2020

* Initial release
* Includes:
    * dclis
    * dclim
    * dclic
    * dclims
    * dclia
    * dclics
