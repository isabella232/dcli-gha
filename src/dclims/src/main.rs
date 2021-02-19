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

use dcli::error::Error;
use dcli::manifestinterface::{FindResult, ManifestInterface};
use dcli::output::Output;
use dcli::utils::{
    determine_data_dir, print_error, print_verbose, EXIT_FAILURE, TSV_DELIM,
    TSV_EOL,
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(verbatim_doc_comment)]
/// Command line tool for searching the Destiny 2 manifest by hash ids.
///
/// Takes a hash / id from the Destiny 2 API, and returns data from the
/// item from the manifest. May return more than one result.
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
    /// Directory where Destiny 2 manifest database file is stored. (optional)
    ///
    /// This will normally be downloaded using the dclim tool, and stored in a file
    /// named manifest.sqlite3 (in the manifest directory specified when running
    /// dclim).
    #[structopt(short = "D", long = "data-dir", parse(from_os_str))]
    data_dir: Option<PathBuf>,

    ///The hash id from the Destiny 2 API for the item to be searched for.
    ///
    ///Example : 326060471
    #[structopt(long = "hash", short = "h", required = true)]
    hash: u32,

    /// Format for command output
    ///
    /// Valid values are default (Default) and tsv.
    ///
    /// tsv outputs in a tab (\t) seperated format of columns with lines
    /// ending in a new line character (\n).
    #[structopt(
        short = "O",
        long = "output-format",
        default_value = "default"
    )]
    output: Output,

    ///Print out additional information
    ///
    ///Output is printed to stderr.
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}

//TODO: can we make has and path reference?
async fn search_manifest_by_hash(
    hash: u32,
    manifest_dir: PathBuf,
) -> Result<Vec<FindResult>, Error> {
    let mut manifest = ManifestInterface::new(&manifest_dir, false).await?;
    let out = manifest.find(hash).await?;

    Ok(out)
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

    let results: Vec<FindResult> =
        match search_manifest_by_hash(opt.hash, data_dir).await {
            Ok(e) => e,
            Err(e) => {
                print_error("Error searching manifest.", e);
                std::process::exit(EXIT_FAILURE);
            }
        };

    match opt.output {
        Output::Default => {
            print_default(results);
        }
        Output::Tsv => {
            print_tsv(results);
        }
    };
}

fn print_default(results: Vec<FindResult>) {
    if results.is_empty() {
        println!("No items found.");
        return;
    }

    let col_w = 15;

    println!(
        "Found {} item{}",
        results.len(),
        if results.len() > 1 { "s" } else { "" }
    );
    println!("-----------------------------");
    for r in results.iter() {
        let default: String = "".to_string();
        let description = r
            .display_properties
            .description
            .as_ref()
            .unwrap_or(&default);
        let icon_path =
            r.display_properties.icon_path.as_ref().unwrap_or(&default);

        println!(
            "{:<0col_w$}{}",
            "Name",
            r.display_properties.name,
            col_w = col_w
        );
        println!("{:<0col_w$}{}", "Description", description, col_w = col_w);
        println!(
            "{:<0col_w$}{}",
            "Has Icon",
            r.display_properties.has_icon,
            col_w = col_w
        );
        println!("{:<0col_w$}{}", "Icon Path", icon_path, col_w = col_w);
        println!();
    }
}

fn print_tsv(results: Vec<FindResult>) {
    if results.is_empty() {
        println!();
        return;
    }

    for (i, r) in results.iter().enumerate() {
        let default: String = "".to_string();
        let description = r
            .display_properties
            .description
            .as_ref()
            .unwrap_or(&default);
        let icon_path =
            r.display_properties.icon_path.as_ref().unwrap_or(&default);

        print!(
            "{i}{delim}{n}{delim}{d}{delim}{hi}{delim}{ip}{eol}",
            i = i,
            n = r.display_properties.name,
            d = description,
            hi = r.display_properties.has_icon,
            ip = icon_path,
            delim = TSV_DELIM,
            eol = TSV_EOL,
        );
    }
}
