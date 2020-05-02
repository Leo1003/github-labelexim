#[macro_use]
extern crate clap;

use clap::{AppSettings, Arg, SubCommand};

mod cmd;

fn main() {
    let matches = app_from_crate!()
        .global_setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::VersionlessSubcommands)
        .subcommand(cmd::subcmd_import())
        .subcommand(cmd::subcmd_export())
        .subcommand(cmd::subcmd_login())
        .subcommand(cmd::subcmd_logout())
        .get_matches();

    match matches.subcommand() {
        ("import", Some(args)) => {},
        ("export", Some(args)) => {},
        ("login", Some(args)) => {},
        ("logout", Some(args)) => {},
        _ => {}
    }
}

