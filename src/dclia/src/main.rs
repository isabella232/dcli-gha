/*
* Copyright 2021 Mike Chambers
* https://github.com/mikechambers/dcli
*
* Permission is hereby granted, free of charge, to any person obtaining a copy of
* this software and associated documentation files (the "Software"), to deal in
* the Software without restriction, including without limitation the rights to
* use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
* of the Software, and to permit persons to whom the Software is furnished to do
* so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
* FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
* COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
* IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
* CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use std::path::PathBuf;

use dcli::apiinterface::ApiInterface;
use dcli::manifest::definitions::{
    ActivityDefinitionData, DestinationDefinitionData, PlaceDefinitionData,
};
//use dcli::error::Error;
use dcli::enums::mode::Mode;
use dcli::enums::platform::Platform;
use dcli::manifestinterface::ManifestInterface;
use dcli::output::Output;
use dcli::response::gpr::CharacterActivitiesData;
use dcli::utils::EXIT_FAILURE;
use dcli::utils::{build_tsv, determine_data_dir, print_error, print_verbose};
use structopt::StructOpt;

const ORBIT_PLACE_HASH: u32 = 2961497387;

#[derive(StructOpt, Debug)]
#[structopt(verbatim_doc_comment)]
/// Command line tool for retrieving current Destiny 2 activity status for player.
///
/// Created by Mike Chambers.
/// https://www.mikechambers.com
///
/// Get support,request features or just chat on the dcli Discord server:
/// https://discord.gg/2Y8bV2Mq3p
///
/// Get the latest version, download the source and log issues at:
/// https://github.com/mikechambers/dcli
///
/// Released under an MIT License.
struct Opt {
    /// Platform for specified id
    ///
    /// Valid values are: xbox, playstation, stadia or steam.
    #[structopt(short = "p", long = "platform", required = true)]
    platform: Platform,

    /// Destiny 2 API member id
    ///
    /// This is not the user name, but the member id retrieved from the Destiny API.
    #[structopt(short = "m", long = "member-id", required = true)]
    member_id: String,

    ///Print out additional information
    ///
    ///Output is printed to stderr.
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Directory where Destiny 2 manifest database file is stored. (optional)
    ///
    /// This will normally be downloaded using the dclim tool, and stored in a file
    /// named manifest.sqlite3 (in the manifest directory specified when running
    /// dclim).
    #[structopt(short = "D", long = "data-dir", parse(from_os_str))]
    data_dir: Option<PathBuf>,

    /// Format for command output
    ///
    /// Valid values are default (Default) and tsv.
    ///
    /// tsv outputs in a tab (\t) seperated format of name / value pairs with lines
    /// ending in a new line character (\n).
    #[structopt(
        short = "O",
        long = "output-format",
        default_value = "default"
    )]
    output: Output,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    print_verbose(&format!("{:#?}", opt), opt.verbose);

    let data_dir = match determine_data_dir(opt.data_dir) {
        Ok(e) => e,
        Err(e) => {
            print_error("Error initializing manifest directory.", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    let client = match ApiInterface::new(opt.verbose) {
        Ok(e) => e,
        Err(e) => {
            print_error("Error initializing API Interface", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    let activities_data: Option<CharacterActivitiesData> = match client
        .retrieve_current_activity(opt.member_id, opt.platform)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            print_error("Error retrieving data from API", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    let activity_data_a = match activities_data {
        Some(e) => e,
        None => {
            match opt.output {
                Output::Default => {
                    println!("Not currently in an activity");
                }
                Output::Tsv => {
                    print_tsv_no_activity();
                }
            };
            return;
        }
    };

    let mut manifest = match ManifestInterface::new(&data_dir, false).await {
        Ok(e) => e,
        Err(e) => {
            print_error("Manifest Error", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    print_verbose(
        &format!(
            "Getting activity definition data from manifest : {}",
            activity_data_a.current_activity_hash
        ),
        opt.verbose,
    );
    let activity_data_m: Option<ActivityDefinitionData> = match manifest
        .get_activity_definition(activity_data_a.current_activity_hash)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            print_error("Error Retrieving Data from Manifest", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    if activity_data_m.is_none() {
        println!("Unknown activity. Make sure you have synced the latest version of the manifest using dclim.");
        return;
    }

    let activity_data_m = activity_data_m.unwrap();

    if activity_data_m.place_hash == ORBIT_PLACE_HASH {
        match opt.output {
            Output::Default => {
                println!("{}", get_in_orbit_human());
            }
            Output::Tsv => {
                print_tsv_orbit();
            }
        };

        return;
    }

    print_verbose(
        &format!(
            "Getting place definition data from manifest : {}",
            activity_data_m.place_hash
        ),
        opt.verbose,
    );
    let place_data_m: Option<PlaceDefinitionData> = match manifest
        .get_place_definition(activity_data_m.place_hash)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            print_error("Error Retrieving Data from Manifest", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    if place_data_m.is_none() {
        println!("Unknown location. Make sure you have synced the latest version of the manifest using dclim.");
        return;
    }
    let place_data_m = place_data_m.unwrap();

    print_verbose(
        &format!(
            "Getting destination definition data from manifest : {}",
            activity_data_m.destination_hash
        ),
        opt.verbose,
    );
    let destination_data_m: Option<DestinationDefinitionData> = match manifest
        .get_destination_definition(activity_data_m.destination_hash)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            print_error("Error Retrieving Data from Manifest", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    if destination_data_m.is_none() {
        println!("Unknown destination. Make sure you have synced the latest version of the manifest using dclim.");
        return;
    }

    let destination_data_m = destination_data_m.unwrap();

    let mut mode = Mode::None;

    //lets find out the mode / activity type name
    print_verbose("Determining activity mode", opt.verbose);
    let activity_type_name: String = match activity_data_a
        .current_activity_mode_type
    {
        // if its set in the API data, we use that
        // this is due to this bug:
        // https://github.com/Bungie-net/api/issues/1341
        Some(e) => {
            mode = e;
            format!("{}", e)
        }
        None => {
            print_verbose(
                &format!(
                    "Activity mode not returned from API. Checking Manifest : {}",
                    activity_data_m.activity_type_hash
                ),
                opt.verbose,
            );
            //otherwise, we go into the manifest to find it
            match manifest
                .get_activity_type_definition(
                    activity_data_m.activity_type_hash,
                )
                .await
            {
                Ok(e) => match e {
                    Some(e) => e.display_properties.name,
                    None => "Unknown".to_string(),
                },
                Err(e) => {
                    print_verbose(
                        &format!(
                            "Activity Mode not found in Manifest : {:?}",
                            e
                        ),
                        opt.verbose,
                    );
                    //Todo: this either means an error, unknown activity, or they are in orbit
                    "Unknown".to_string()
                }
            }
        }
    };

    // note if for some reason correct activities are not displayed for some
    // crucible modes, then this may be false (i've only seen this as an issue
    // for raids thought (see link above for bug (raid)))
    let description = activity_data_m
        .display_properties
        .description
        .unwrap_or_else(|| "".to_string());
    let activity_name = activity_data_m.display_properties.name;
    let place_name = place_data_m.display_properties.name;
    let destination_name = destination_data_m.display_properties.name;

    match opt.output {
        Output::Default => {
            print_default(
                mode,
                &activity_type_name,
                &activity_name,
                &place_name,
                &destination_name,
                &description,
            );
        }
        Output::Tsv => {
            print_tsv(
                mode,
                &activity_type_name,
                &activity_name,
                &place_name,
                &destination_name,
                &description,
                true,
            );
        }
    };
}

fn print_tsv_orbit() {
    print_tsv(Mode::None, "", "", "Orbit", "", "", true);
}

fn print_tsv_no_activity() {
    print_tsv(Mode::None, "", "", "", "", "", false);
}

fn print_tsv(
    mode: Mode,
    activity_type_name: &str,
    activity_name: &str,
    place_name: &str,
    destination_name: &str,
    description: &str,
    in_activity: bool,
) {
    //figure out if they are in orbit since bungie doesnt give us
    //a mode for it
    let human_status = if mode == Mode::None && in_activity {
        get_in_orbit_human()
    } else {
        build_human_status(
            mode,
            activity_type_name,
            activity_name,
            place_name,
            destination_name,
            description,
        )
    };

    let mut name_values: Vec<(&str, String)> = Vec::new();

    name_values.push(("in_activity", in_activity.to_string()));
    name_values.push(("activity_type_name", activity_type_name.to_string()));
    name_values.push(("activity_name", activity_name.to_string()));
    name_values.push(("place_name", place_name.to_string()));
    name_values.push(("destination_name", destination_name.to_string()));
    name_values.push(("description", description.to_string()));
    name_values.push(("human_status", human_status));
    name_values.push(("is_crucible", mode.is_crucible().to_string()));

    print!("{}", build_tsv(name_values));
}

fn print_default(
    mode: Mode,
    activity_type_name: &str,
    activity_name: &str,
    place_name: &str,
    _destination_name: &str,
    description: &str,
) {
    let out = build_human_status(
        mode,
        activity_type_name,
        activity_name,
        place_name,
        _destination_name,
        description,
    );

    println!("{}", out);
}

fn build_human_status(
    mode: Mode,
    activity_type_name: &str,
    activity_name: &str,
    place_name: &str,
    _destination_name: &str,
    description: &str,
) -> String {
    if mode == Mode::Patrol {
        format!("Exploring on {}", place_name)
    } else if mode.is_gambit() || mode.is_crucible() {
        format!(
            "Playing {} on {} ({})",
            activity_type_name, activity_name, description
        )
    } else if mode == Mode::Strike {
        format!(
            "Running {} {} on {}",
            activity_name, activity_type_name, place_name
        )
    } else if mode == Mode::Social {
        format!("Hanging out in the {} on {}", activity_name, place_name)
    } else if mode == Mode::Story {
        format!("Playing {} story on {}", activity_name, place_name)
    } else if mode.is_nightfall() {
        format!(
            "Playing {} {} on {}",
            description, activity_name, place_name
        )
    } else {
        format!(
            "Playing {} {} on {}",
            activity_name, activity_type_name, place_name
        )
    }
}

fn get_in_orbit_human() -> String {
    "Currently sitting in Orbit".to_string()
}
