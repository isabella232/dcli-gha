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

use std::str::FromStr;
use std::{collections::HashMap, path::PathBuf};

use dcli::{
    apiinterface::ApiInterface,
    crucible::{
        AggregateCruciblePerformances, CrucibleActivity,
        CruciblePlayerPerformance, Player,
    },
    enums::completionreason::CompletionReason,
    utils::{calculate_avg, f32_are_equal},
};
use dcli::{enums::platform::Platform, utils::truncate_ascii_string};

use dcli::enums::mode::Mode;
use dcli::manifestinterface::ManifestInterface;

use dcli::enums::character::CharacterClassSelection;
use dcli::error::Error;

use dcli::activitystoreinterface::ActivityStoreInterface;

use dcli::utils::{
    determine_data_dir, format_f32, human_date_format, human_duration,
    repeat_str,
};

use dcli::utils::EXIT_FAILURE;
use dcli::utils::{print_error, print_verbose};
use structopt::StructOpt;

const ELO_SCALE: f32 = 10.0;

fn parse_and_validate_mode(src: &str) -> Result<Mode, String> {
    let mode = Mode::from_str(src)?;

    if !mode.is_crucible() {
        return Err(format!("Unsupported mode specified : {}", src));
    }

    Ok(mode)
}

fn generate_score(data: &CrucibleActivity) -> String {
    let mut tokens: Vec<String> = Vec::new();

    for t in data.teams.values() {
        tokens.push(t.score.to_string());
        tokens.push("-".to_string());
    }

    tokens.pop();

    tokens.join("")
}

async fn get_combat_ratings(
    data: &CrucibleActivity,
    verbose: bool,
) -> HashMap<u64, f32> {
    let mut players: Vec<&Player> = Vec::new();

    for t in data.teams.values() {
        for p in &t.player_performances {
            players.push(&p.player);
        }
    }

    let elo_hash: HashMap<u64, f32> = match ApiInterface::new(verbose) {
        Ok(e) => {
            let mut player_refs: Vec<&Player> = Vec::new();
            for t in data.teams.values() {
                for p in &t.player_performances {
                    player_refs.push(&p.player);
                }
            }

            match e
                .retrieve_combat_ratings(&player_refs, &data.details.mode)
                .await
            {
                Ok(e) => e,
                Err(_e) => HashMap::new(),
            }
        }
        Err(_e) => HashMap::new(),
    };
    elo_hash
}

fn print_default(
    data: &CrucibleActivity,
    elo_hash: &HashMap<u64, f32>,
    member_id: &str,
    details: bool,
    weapon_count: u32,
    verbose: bool,
) {
    let col_w = 8;
    let name_col_w = 24;

    let mut activity_duration = "".to_string();
    let mut completion_reason = "".to_string();
    let mut standing_str = "".to_string();

    if let Some(e) = data.get_member_performance(member_id) {
        completion_reason =
            if e.stats.completion_reason == CompletionReason::Unknown {
                "".to_string()
            } else {
                format!("({})", e.stats.completion_reason)
            };

        activity_duration =
            format!("({})", human_duration(e.stats.activity_duration_seconds));
        standing_str = format!("{}!", e.stats.standing);
    };

    let team_title_border = repeat_str("-", name_col_w + col_w);
    let activity_title_border = repeat_str("=", name_col_w + col_w + col_w);

    println!();
    println!("ACTIVITY");
    println!("{}", activity_title_border);

    println!(
        "{} on {} :: {} {}",
        data.details.mode,
        data.details.map_name,
        human_date_format(&data.details.period),
        activity_duration
    );

    if verbose {
        println!("Activity ID : {}", data.details.id);
    }

    println!("{}", standing_str);
    println!("{} {}", generate_score(data), completion_reason);

    println!();

    let header = format!("{:<0name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}",
    "PLAYER",
    "KILLS",
    "ASTS",
    "K+A",
    "DEATHS",
    "K/D",
    "KD/A",
    "EFF",
    "SUP",
    "GREN",
    "MEL",
    "MED",
    "RATING",
    "STATUS",
    col_w=col_w,
    name_col_w = name_col_w,
    );

    let table_width = header.chars().count();
    let header_border = repeat_str("=", table_width);
    let entry_border = repeat_str(".", table_width);
    let footer_border = repeat_str("-", table_width);

    let mut all_performances: Vec<&CruciblePlayerPerformance> = Vec::new();
    let mut elo_total_count = 0;
    let mut elo_total_total = 0.0;
    for v in data.teams.values() {
        let mut elo_team_count = 0;
        let mut elo_team_total = 0.0;

        println!("[{}] {} Team {}!", v.score, v.display_name, v.standing);
        println!("{}", team_title_border);
        println!("{}", header);
        println!("{}", header_border);

        let mut first_performance = true;

        let mut player_performances = v.player_performances.clone();
        player_performances.sort_by(|a, b| {
            b.stats.opponents_defeated.cmp(&a.stats.opponents_defeated)
        });

        for p in &player_performances {
            let elo = *elo_hash.get(&p.player.calculate_hash()).unwrap_or(&0.0)
                * ELO_SCALE;

            let mut elo_str = "".to_string();
            if !f32_are_equal(elo, 0.0) {
                elo_team_count += 1;
                elo_team_total += elo;

                elo_total_count += 1;
                elo_total_total += elo;

                elo_str = format_f32(elo, 0);
            }

            let extended = p.stats.extended.as_ref().unwrap();
            println!("{:<0name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}",
                truncate_ascii_string(&p.player.display_name, name_col_w),
                p.stats.kills.to_string(),
                p.stats.assists.to_string(),
                p.stats.opponents_defeated.to_string(),
                p.stats.deaths.to_string(),
                format_f32(p.stats.kills_deaths_ratio, 2),
                format_f32(p.stats.kills_deaths_assists, 2),
                format_f32(p.stats.efficiency, 2),
                extended.weapon_kills_super.to_string(),
                extended.weapon_kills_grenade.to_string(),
                extended.weapon_kills_ability.to_string(),
                extended.all_medals_earned.to_string(),
                elo_str,
                p.stats.generate_status(),
                col_w=col_w,
                name_col_w = name_col_w,
            );

            //todo: what if they dont have weapon kills (test)
            if details && !extended.weapons.is_empty() {
                println!("{}", entry_border);

                let mut weapons = extended.weapons.clone();
                weapons.sort_by(|a, b| b.kills.cmp(&a.kills));

                let mut min_index = 2;
                if first_performance {
                    println!(
                        //"{:>0w_name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w2$}",
                        "{:<0col_w$}{:>0w_name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w2$}",
                        format!("{}", p.player.class_type),
                        "NAME",
                        "KILLS",
                        "PREC",
                        "%",
                        "TYPE",
                        w_name_col_w = col_w + col_w + name_col_w,
                        col_w = col_w,
                        col_w2 = col_w * 3,
                    );
                    first_performance = false;
                    min_index = 1;
                }

                for i in 0..std::cmp::max(min_index, weapons.len()) {
                    let modifier = 2 - min_index;
                    let meta = match i + modifier {
                        0 => format!("{}", p.player.class_type),
                        1 => p.player.light_level.to_string(),
                        _ => "".to_string(),
                    };

                    let mut weapon_name = "".to_string();
                    let mut weapon_kills = "".to_string();
                    let mut precision_kills = "".to_string();
                    let mut precision_kills_percent = "".to_string();
                    let mut weapon_type = "".to_string();

                    if i < weapons.len() {
                        let w = &weapons[i];
                        weapon_name = w.weapon.name.to_string();
                        weapon_kills = w.kills.to_string();
                        precision_kills = w.precision_kills.to_string();
                        precision_kills_percent =
                            format_f32(w.precision_kills_percent * 100.0, 0)
                                .to_string();
                        weapon_type = format!("{}", w.weapon.item_sub_type);
                    }

                    println!(
                        "{:<0col_w$}{:>0w_name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w2$}",
                        meta,
                        weapon_name,
                        weapon_kills,
                        precision_kills,
                        precision_kills_percent,
                        weapon_type,
                        w_name_col_w = col_w + col_w + name_col_w,
                        col_w = col_w,
                        col_w2 = col_w * 3,
                    );
                }
                println!();
            }
        }
        println!("{}", footer_border);

        let mut cpp: Vec<&CruciblePlayerPerformance> = Vec::new();

        for p in &v.player_performances {
            cpp.push(p);
            all_performances.push(p);
        }

        let aggregate = AggregateCruciblePerformances::with_performances(&cpp);

        let agg_extended = aggregate.extended.as_ref().unwrap();
        let agg_supers = agg_extended.weapon_kills_super;
        let agg_grenades = agg_extended.weapon_kills_grenade;
        let agg_melees = agg_extended.weapon_kills_melee;

        let team_elo = calculate_avg(elo_team_total, elo_team_count);
        let team_elo_str = if f32_are_equal(team_elo, 0.0) {
            "".to_string()
        } else {
            format_f32(team_elo, 0)
        };

        println!("{:<0name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}",
            "TOTAL",
            aggregate.kills.to_string(),
            aggregate.assists.to_string(),
            aggregate.opponents_defeated.to_string(),
            aggregate.deaths.to_string(),
            format_f32(aggregate.kills_deaths_ratio, 2),
            format_f32(aggregate.kills_deaths_assists, 2),
            format_f32(aggregate.efficiency, 2),
            agg_supers.to_string(),
            agg_grenades.to_string(),
            agg_melees.to_string(),
            aggregate.extended.as_ref().unwrap().all_medals_earned.to_string(),
            "",
            "",
            col_w=col_w,
            name_col_w = name_col_w,
        );

        println!("{:<0name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}",
            "AVG",
            format_f32(aggregate.kills as f32 / player_performances.len() as f32, 2),
            format_f32(aggregate.assists as f32 / player_performances.len() as f32,2),
            format_f32(aggregate.opponents_defeated as f32 / player_performances.len() as f32,2),
            format_f32(aggregate.deaths as f32 / player_performances.len() as f32,2),
            "",
            "",
            "",
            format_f32(agg_supers as f32 / player_performances.len() as f32,2),
            format_f32(agg_grenades as f32 / player_performances.len() as f32,2),
            format_f32(agg_melees as f32 / player_performances.len() as f32,2),
            format_f32(aggregate.extended.as_ref().unwrap().all_medals_earned as f32 / player_performances.len() as f32,2),
            team_elo_str,
            "", //MAKE THIS REASON FOR COMPLETEION
            col_w=col_w,
            name_col_w = name_col_w,
        );

        //println!("{}", header_border);
        //println!("{}", header);
        println!();
    }

    println!("Combined");
    println!("{}", team_title_border);

    let aggregate =
        AggregateCruciblePerformances::with_performances(&all_performances);

    let agg_extended = aggregate.extended.as_ref().unwrap();
    let agg_supers = agg_extended.weapon_kills_super;
    let agg_grenades = agg_extended.weapon_kills_grenade;
    let agg_melees = agg_extended.weapon_kills_melee;

    println!("{}", header);
    println!("{}", header_border);
    println!("{:<0name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}",
        "TOTAL",
        aggregate.kills.to_string(),
        aggregate.assists.to_string(),
        aggregate.opponents_defeated.to_string(),
        aggregate.deaths.to_string(),
        format_f32(aggregate.kills_deaths_ratio, 2),
        format_f32(aggregate.kills_deaths_assists, 2),
        format_f32(aggregate.efficiency, 2),
        agg_supers.to_string(),
        agg_grenades.to_string(),
        agg_melees.to_string(),
        aggregate.extended.as_ref().unwrap().all_medals_earned.to_string(),
        "",
        "", //MAKE THIS REASON FOR COMPLETEION
        col_w=col_w,
        name_col_w = name_col_w,
    );

    let total_elo = calculate_avg(elo_total_total, elo_total_count);
    let total_elo_str = if f32_are_equal(total_elo, 0.0) {
        "".to_string()
    } else {
        format_f32(total_elo, 0)
    };

    println!("{:<0name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}",
    "AVG",
    format_f32(aggregate.kills as f32 / all_performances.len() as f32, 2),
    format_f32(aggregate.assists as f32 / all_performances.len() as f32,2),
    format_f32(aggregate.opponents_defeated as f32 / all_performances.len() as f32,2),
    format_f32(aggregate.deaths as f32 / all_performances.len() as f32,2),
    "",
    "",
    "",
    format_f32(agg_supers as f32 / all_performances.len() as f32,2),
    format_f32(agg_grenades as f32 / all_performances.len() as f32,2),
    format_f32(agg_melees as f32 / all_performances.len() as f32,2),
    format_f32(aggregate.extended.as_ref().unwrap().all_medals_earned as f32 / all_performances.len() as f32,2),
    total_elo_str,
    "", //MAKE THIS REASON FOR COMPLETEION
    col_w=col_w,
    name_col_w = name_col_w,
);

    println!();

    let wep_col = name_col_w + col_w;
    let wep_header_str = format!(
        "{:<0name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0name_col_w$}",
        "WEAPON",
        "KILLS",
        "% TOTAL",
        "PREC",
        "% PREC",
        "TYPE",
        col_w = col_w,
        name_col_w = wep_col,
    );

    let wep_divider = repeat_str(&"=", wep_header_str.chars().count());
    println!("{}", wep_header_str);
    println!("{}", wep_divider);

    let weapons = &aggregate.extended.as_ref().unwrap().weapons;
    let max_weps = std::cmp::min(weapon_count as usize, weapons.len());

    let wep_col = name_col_w + col_w;
    for w in &weapons[..max_weps] {
        println!(
            "{:<0name_col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0col_w$}{:>0name_col_w$}",
            w.weapon.name,
            w.kills.to_string(),
            format!(
                "{}%",
                format_f32((w.kills as f32 / aggregate.kills as f32) * 100.0, 2)
            ),
            w.precision_kills.to_string(),
            format!("{}%", format_f32(w.precision_kills_percent, 2)),
            format!("{}", w.weapon.item_sub_type),
            col_w = col_w,
            name_col_w = wep_col,
        );
    }

    println!();
    println!("STATUS : L - Joined late, E - Left early");
    println!();
}

#[derive(StructOpt, Debug)]
#[structopt(verbatim_doc_comment)]
/// Command line tool for retrieving and viewing Destiny 2 Crucible activity details.
///
/// By default the details on the last activity will be displayed, and you can
/// specify the specific activity via the --activity-index argument. The index
/// can be retrieved from dcliah, as well as directly from the sqlite datastore
/// (activity.id)
///
/// Created by Mike Chambers.
/// https://www.mikechambers.com
///
/// Get support, request features or just chat on the dcli Discord server:
/// https://discord.gg/2Y8bV2Mq3p
///
/// Get the latest version, download the source and log issues at:
/// https://github.com/mikechambers/dcli
///
/// Released under an MIT License.
struct Opt {
    /// Destiny 2 API member id
    ///
    /// This is not the user name, but the member id retrieved from the Destiny API.
    #[structopt(short = "m", long = "member-id", required = true)]
    member_id: String,

    /// Platform for specified id
    ///
    /// Valid values are: xbox, playstation, stadia or steam.
    #[structopt(short = "p", long = "platform", required = true)]
    platform: Platform,

    /// Activity mode from which to return last activity
    ///
    /// Supported values are all_pvp (default), control, clash, elimination,
    /// mayhem, iron_banner, all_private, rumble, pvp_competitive,
    /// quickplay and trials_of_osiris.
    ///
    /// Addition values available are crimsom_doubles, supremacy, survival,
    /// countdown, all_doubles, doubles, private_clash, private_control,
    /// private_survival, private_rumble, showdown, lockdown,
    /// scorched, scorched_team, breakthrough, clash_quickplay, trials_of_the_nine
    #[structopt(long = "mode", short = "M", 
        parse(try_from_str=parse_and_validate_mode), default_value = "all_pvp")]
    mode: Mode,

    /// Character class to retrieve data for
    ///
    /// Valid values include hunter, titan, warlock, last_active and all.
    #[structopt(short = "C", long = "class", default_value = "last_active")]
    character_class_selection: CharacterClassSelection,

    ///Print out additional information
    ///
    ///Output is printed to stderr.
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Don't sync activities
    ///
    /// If flag is set, activities will not be retrieved before displaying stats.
    /// This is useful in case you are syncing activities in a seperate process.
    #[structopt(short = "N", long = "no-sync")]
    no_sync: bool,

    /// Display extended activity details
    ///
    /// If flag is set, additional information will be displayed, including per
    /// user weapon stats.
    #[structopt(short = "d", long = "details")]
    details: bool,

    /// The number of weapons to display details for
    #[structopt(long = "weapon-count", short = "w", default_value = "5")]
    weapon_count: u32,

    /// The index of the activity to display data about
    ///
    /// By default, the last activity will be displayed. The index can be retrieved
    /// from other dcli apps, such as dcliah, or directly from the sqlite datastore.
    #[structopt(long = "activity-index", short = "a")]
    activity_index: Option<u32>,

    /// Directory where Destiny 2 manifest and activity database files are stored. (optional)
    ///
    /// This will normally be downloaded using the dclim and dclias tools, and uses
    /// a system appropriate directory by default.
    #[structopt(short = "D", long = "data-dir", parse(from_os_str))]
    data_dir: Option<PathBuf>,
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

    let mut store =
        match ActivityStoreInterface::init_with_path(&data_dir, opt.verbose)
            .await
        {
            Ok(e) => e,
            Err(e) => {
                print_error(
                    "Could not initialize activity store. Have you run dclias?",
                    e,
                );
                std::process::exit(EXIT_FAILURE);
            }
        };

    let mut manifest = match ManifestInterface::new(&data_dir, false).await {
        Ok(e) => e,
        Err(e) => {
            print_error(
                "Could not initialize manifest. Have you run dclim?",
                e,
            );
            std::process::exit(EXIT_FAILURE);
        }
    };

    if !opt.no_sync {
        match store.sync(&opt.member_id, &opt.platform).await {
            Ok(_e) => (),
            Err(e) => {
                eprintln!("Could not sync activity store {}", e);
                eprintln!("Using existing data");
            }
        };
    }

    let data_result = match opt.activity_index {
        Some(e) => store.retrieve_activity_by_index(e, &mut manifest).await,
        None => {
            store
                .retrieve_last_activity(
                    &opt.member_id,
                    &opt.platform,
                    &opt.character_class_selection,
                    &opt.mode,
                    &mut manifest,
                )
                .await
        }
    };

    let data = match data_result {
        Ok(e) => e,
        Err(e) => {
            if e == Error::ActivityNotFound {
                println!("No activities found");
                return;
            }

            print_error("Could not retrieve data from activity store.", e);
            std::process::exit(EXIT_FAILURE);
        }
    };

    let elo_hash = get_combat_ratings(&data, opt.verbose).await;

    print_default(
        &data,
        &elo_hash,
        &opt.member_id,
        opt.details,
        opt.weapon_count,
        opt.verbose,
    );
}
