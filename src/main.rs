#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate clap;

use anyhow::{Context as _, Result as AnyResult};
use clap::{AppSettings, ArgMatches};
use tokio::runtime::Runtime;

use config::Config;

mod api;
mod cmd;
mod config;

fn main() -> AnyResult<()> {
    let matches = app_from_crate!()
        .global_setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(cmd::subcmd_import())
        .subcommand(cmd::subcmd_export())
        .subcommand(cmd::subcmd_login())
        .subcommand(cmd::subcmd_logout())
        .get_matches();

    let mut rt = Runtime::new().expect("Cannot spawn runtime");

    let fut = match matches.subcommand() {
        ("import", Some(args)) => rt.spawn(main_import(args.clone())),
        ("export", Some(args)) => rt.spawn(main_export(args.clone())),
        ("login", Some(args)) => rt.spawn(main_login(args.clone())),
        ("logout", Some(args)) => rt.spawn(main_logout(args.clone())),
        _ => unreachable!("Unknown command"),
    };

    rt.block_on(fut).unwrap()
}

async fn main_import(args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let cfg = Config::load(&cfg_path)
        .await
        .context("Cannot load config")?;

    todo!("upload to github");

    Ok(())
}

async fn main_export(args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let cfg = Config::load_or_create(&cfg_path)
        .await
        .context("Cannot load config")?;

    let client = api::GithubClient::new(&cfg.token);
    if let Some((owner, repo)) = api::parse_github_repo(args.value_of("REPO").unwrap()) {
        let labels = client.get_labels(owner, repo).await?;
        dbg!(labels);
        //TODO: Save data
    } else {
        bail!("Failed to parse the url");
    }

    Ok(())
}

async fn main_login(args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let mut cfg = Config::load_or_create(&cfg_path)
        .await
        .context("Cannot load config")?;

    cfg.token = args.value_of("TOKEN").unwrap().to_owned();

    cfg.save(&cfg_path).await?;

    Ok(())
}

async fn main_logout(_args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let mut cfg = Config::load_or_create(&cfg_path)
        .await
        .context("Cannot load config")?;

    cfg.token.clear();

    cfg.save(&cfg_path).await?;

    Ok(())
}
