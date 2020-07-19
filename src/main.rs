mod filename_decoder;
mod zip_central_directory;
mod zip_eocd;
mod zip_error;
use ansi_term::Color::{Green, Red};
use anyhow::anyhow;
use clap::{App, Arg};
use std::fs::File;
use std::io::BufReader;
use zip_central_directory::ZipCDEntry;
use zip_eocd::ZipEOCD;

#[derive(thiserror::Error, Debug)]
enum InvalidArgument {
    #[error("no argument <{arg_name}> was passed")]
    NoArgument { arg_name: String },
}

fn main() -> anyhow::Result<()> {
    let app = App::new("zfu")
        .arg(
            Arg::with_name("input")
                .about("Path to the ZIP file where you want to change the encoding of the file name to UTF-8")
                .required(true)
            )
        .arg(
            Arg::with_name("check")
                .long("check")
                .short('c')
                .about("Finds out if its file names are encoded in UTF-8.")
        )
        .arg(
            Arg::with_name("list")
                .long("list")
                .about("Displays the list of file names in the ZIP archive.")
        )
        .arg(
            Arg::with_name("language")
                .long("language")
                .short('l')
                .about("Specifys the language of file names in the ZIP archive.")
        )
        .arg(
            Arg::with_name("utf-8")
                .long("utf8")
                .short('u')
                .about("Treats the encoding of the ZIP archive as UTF-8.")
        );

    let matches = app.get_matches();
    let mut zip_file = match matches.value_of("input") {
        None => {
            return Err(InvalidArgument::NoArgument {
                arg_name: "input".to_string(),
            }
            .into());
        }
        Some(a) => BufReader::new(File::open(a)?),
    };

    if !matches.is_present("check") && !matches.is_present("list") {
        return Err(anyhow!(
            "Sorry without check mode has not yet been implemented.  Add {} or {} option to the arguments.", Green.bold().paint("-c").to_string(), Green.bold().paint("--list").to_string()
        ));
    }

    let eocd = ZipEOCD::from_reader(&mut zip_file)?;
    eocd.check_unsupported_zip_type()?;
    let sjis_decoder = filename_decoder::FileNameDecoder::init(None, false);
    let utf8_decoder = filename_decoder::FileNameDecoder::init(None, true);

    let cd_entries = ZipCDEntry::all_from_eocd(&mut zip_file, &eocd)?;

    if matches.is_present("check") {
        let utf8_entries_count = cd_entries
            .iter()
            .filter(|&cd| cd.is_encoded_in_utf8())
            .count();
        if utf8_entries_count == eocd.n_cd_entries as usize {
            println!(
                "{}",
                Green
                    .bold()
                    .paint("All file names are explicitly encoded in UTF-8.")
            );
            return Ok(());
        }
        if utf8_entries_count > 0 {
            println!(
                "{}",
                Red.bold().paint(format!(
                    "Some file names are not explicitly encoded in UTF-8. ({} / {})",
                    eocd.n_cd_entries as usize - utf8_entries_count,
                    eocd.n_cd_entries
                ))
            );
            std::process::exit(1);
        }
        println!(
            "{}",
            Red.bold()
                .paint("All file names are not explicitly encoded in UTF-8.")
        );
        std::process::exit(1);
    } else if matches.is_present("list") {
        for cd in cd_entries {
            if cd.is_encoded_in_utf8() {
                println!(
                    "{}:{}:{}",
                    Green.bold().paint("EXPLICIT"),
                    Green.bold().paint("UTF-8"),
                    utf8_decoder.to_string_lossy(&cd.file_name_raw)
                );
            } else {
                println!(
                    "{}:{}:{}",
                    Red.bold().paint("GUESSED"),
                    Red.bold().paint("SJIS"),
                    sjis_decoder.to_string_lossy(&cd.file_name_raw)
                );
            }
        }
    }
    return Ok(());
}
