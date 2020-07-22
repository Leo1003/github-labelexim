#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use anyhow::{Context as _, Result as AnyResult};
use clap::{AppSettings, ArgMatches};
use tokio::prelude::*;
use tokio::runtime::Runtime;
use std::collections::HashMap;
use unicase::UniCase;

use api::Label;
use config::Config;

mod api;
mod cmd;
mod config;

fn main() -> AnyResult<()> {
    env_logger::init();

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

    let opt_override = args.is_present("override");

    let client = api::GithubClient::new(&cfg.token);
    if let Some((owner, repo)) = api::parse_github_repo(args.value_of("REPO").unwrap()) {
        // Read labels file
        let json = if let Some(file_path) = args.value_of("file") {
            let mut f = tokio::fs::File::open(file_path).await?;
            let mut json = String::new();
            f.read_to_string(&mut json).await?;
            json
        } else {
            // Not specifying the output file, write to stdout
            let mut json = String::new();
            tokio::io::stdin().read_to_string(&mut json).await?;
            json
        };

        let labels = serde_json::from_str::<Vec<Label>>(&json)
            .with_context(|| "Failed to parse the input into labels!")?;

        let cur_labels = client.get_labels(owner, repo).await?;
        let label_map = cur_labels.into_iter().map(|l| {
            (UniCase::ascii(l.name.clone()), l)
        }).collect::<HashMap<UniCase<String>, Label>>();

        if !opt_override {
            // Update or create labels
            for l in &labels {
                if let Some((key, _)) = label_map.get_key_value(&UniCase::ascii(l.name.clone())) {
                    info!("Updating label: {}", &l.name);
                    client.update_label_with_name(owner, repo, key, &l).await?;
                } else {
                    info!("Creating label: {}", &l.name);
                    client.new_label(owner, repo, &l).await?;
                }
            }
        } else {
            warn!("Running in override mode, removing all label...");
            // Delete all labels
            for name in label_map.keys() {
                info!("Removing label: {}", name);
                client.remove_label(owner, repo, name).await?;
            }
            // Create all labels
            for l in &labels {
                info!("Creating label: {}", &l.name);
                client.new_label(owner, repo, &l).await?;
            }
        }
    } else {
        bail!("Failed to parse the url");
    }

    info!("Import completed!");
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
        let json = serde_json::to_string_pretty(&labels)?;

        if let Some(output_path) = args.value_of("file") {
            let mut f = tokio::fs::File::create(output_path).await?;
            f.write_all(json.as_bytes()).await?;
        } else {
            // Not specifying the output file, write to stdout
            tokio::io::stdout().write_all(json.as_bytes()).await?;
        }
    } else {
        bail!("Failed to parse the url");
    }

    info!("Export completed!");
    Ok(())
}

async fn main_login(args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let mut cfg = Config::load_or_create(&cfg_path)
        .await
        .context("Cannot load config")?;

    cfg.token = args.value_of("TOKEN").unwrap().to_owned();

    info!("Checking Github token...");
    let client = api::GithubClient::new(&cfg.token);
    client.check_token().await
        .with_context(|| "The token is invalid")?;

    cfg.save(&cfg_path).await?;
    info!("Token saved.");

    Ok(())
}

async fn main_logout(_args: ArgMatches<'static>) -> AnyResult<()> {
    let cfg_path = config::default_config_path();
    let mut cfg = Config::load_or_create(&cfg_path)
        .await
        .context("Cannot load config")?;

    cfg.token.clear();

    cfg.save(&cfg_path).await?;
    info!("Token cleared.");

    Ok(())
}
