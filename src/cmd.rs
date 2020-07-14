use clap::{App, Arg, SubCommand};

pub fn subcmd_import() -> App<'static, 'static> {
    SubCommand::with_name("import")
        .about("Import labels from a JSON format data")
        .arg(
            Arg::with_name("override")
                .long("override")
                .help("remove all label before upload. Use with caution!!!"),
        )
        .arg(
            Arg::with_name("file")
                .long("file")
                .short("f")
                .help("specify JSON format labels file. If not specified, will read from stdin.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("REPO")
                .required(true)
                .help("URL or \"author/repo\" to be imported to.")
                .index(1),
        )
}

pub fn subcmd_export() -> App<'static, 'static> {
    SubCommand::with_name("export")
        .about("Export a repository's labels to a JSON format data")
        .arg(
            Arg::with_name("file")
                .long("file")
                .short("f")
                .help("specify JSON format labels file. If not specified, will write to stdout.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("REPO")
                .required(true)
                .help("URL or \"author/repo\" to be imported to.")
                .index(1),
        )
}

pub fn subcmd_login() -> App<'static, 'static> {
    SubCommand::with_name("login")
        .about("Save Github personal access token into the config file")
        .arg(
            Arg::with_name("TOKEN")
                .required(true)
                .help("personal access token to used")
                .index(1),
        )
}

pub fn subcmd_logout() -> App<'static, 'static> {
    SubCommand::with_name("logout").about("Remove previous saved login token")
}
