#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate clap;

use anyhow::{Result as AnyResult, Context as _};
use clap::{AppSettings, Arg, ArgMatches, SubCommand};
use tokio::runtime::Runtime;

use config::Config;

mod cmd;
mod config;

fn main() -> AnyResult<()> {
    let matches = app_from_crate!()
        .global_setting(AppSettings::SubcommandRequiredElseHelp)
        .global_setting(AppSettings::VersionlessSubcommands)
        .subcommand(cmd::subcmd_import())
        .subcommand(cmd::subcmd_export())
        .subcommand(cmd::subcmd_login())
        .subcommand(cmd::subcmd_logout())
        .get_matches();

    let mut rt = Runtime::new().expect("Cannot spawn runtime");

    let fut = match matches.subcommand() {
        ("import", Some(args)) => tokio::spawn(main_import(args.clone())),
        ("export", Some(args)) => tokio::spawn(main_export(args.clone())),
        ("login", Some(args)) => tokio::spawn(main_login(args.clone())),
        ("logout", Some(args)) => tokio::spawn(main_logout(args.clone())),
        _ => unreachable!("Unknown command"),
    };

    rt.block_on(fut).unwrap()
}

async fn main_import(args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let cfg = Config::load_or_create(&cfg_path).await
        .context("Cannot load config")?;
    if cfg.token.is_empty() {
        bail!("Empty token");
    }

    todo!("upload to github");

    Ok(())
}

async fn main_export(args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let cfg = Config::load_or_create(&cfg_path).await
        .context("Cannot load config")?;

    todo!("download from github");

    Ok(())
}

async fn main_login(args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let mut cfg = Config::load_or_create(&cfg_path).await
        .context("Cannot load config")?;

    cfg.token = args.value_of("TOKEN").unwrap().to_owned();

    cfg.save(&cfg_path).await?;

    Ok(())
}

async fn main_logout(_args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let mut cfg = Config::load_or_create(&cfg_path).await
        .context("Cannot load config")?;

    cfg.token.clear();

    cfg.save(&cfg_path).await?;

    Ok(())
}
