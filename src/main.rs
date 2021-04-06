use octocrab::Octocrab;
use rand::prelude::*;
use serde::*;
use std::default::Default;
use structopt::StructOpt;

mod git;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub personal_token: String,
    pub base_url: String,
    pub repo_username: String,
    pub repo_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            personal_token: "ghp_change_this_token".to_string(),
            base_url: "https://your.site.com/path/".to_string(),
            repo_username: "your_github_username".to_string(),
            repo_name: "your_repo_name".to_string(),
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "sls", about = "static link shortener")]
struct Opt {
    #[structopt(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, StructOpt)]
enum Subcommand {
    Init,
    Create(Create),
    Update(Update),
    Delete(Delete),
}

#[derive(Debug, StructOpt)]
struct Create {
    #[structopt(long, short)]
    name: Option<String>,
    url: String,
}

#[derive(Debug, StructOpt)]
struct Update {
    name: String,
    url: String,
}

#[derive(Debug, StructOpt)]
struct Delete {
    name: String,
}

fn name_from_maybe_url<'a>(name: &'a str, config: &Config) -> &'a str {
    // Remove base url
    let name = name.strip_prefix(&config.base_url).unwrap_or(&name);

    // Remove any trailing slashes
    name.trim_end_matches('/')
}

async fn subcommand_create(Create { name, url }: Create, octocrab: &Octocrab, config: &Config) {
    let name = name.unwrap_or_else(|| {
        const IDENT_LEN: usize = 8;

        // TODO: make this const too, if possible
        let alphabet = "abcdefghijklmnopqrstuvwxyz";
        let alphabet_chars = alphabet.chars().collect::<Vec<char>>();

        let mut rng = rand::thread_rng();

        (0..IDENT_LEN)
            .map(|_| alphabet_chars.choose(&mut rng).unwrap())
            .collect::<String>()
    });

    let shortlink = git::Shortlink {
        name: &name,
        url: &url,
        sha: None,
    };
    let shortlink_url = shortlink.send(octocrab, config).await;

    println!(
        "successfully created shortlink \"{}\" from {} to {}",
        name, shortlink_url, url
    );
}

async fn subcommand_update(Update { name, url }: Update, octocrab: &Octocrab, config: &Config) {
    let name = name_from_maybe_url(&name, config);

    let sha = git::get_sha(name, octocrab, config).await;

    let shortlink = git::Shortlink {
        name,
        url: &url,
        sha: Some(&sha),
    };
    let shortlink_url = shortlink.send(octocrab, config).await;

    println!(
        "successfully updated shortlink \"{}\" from {} to {}",
        name, shortlink_url, url
    );
}

async fn subcommand_delete(Delete { name }: Delete, octocrab: &Octocrab, config: &Config) {
    let name = name_from_maybe_url(&name, config);

    let sha = git::get_sha(name, octocrab, config).await;
    git::delete(name, sha, octocrab, config).await;

    println!("successfully removed shortlink \"{}\"", name);
}

#[tokio::main]
async fn main() {
    let config: Config = confy::load("sls").unwrap();

    let octocrab = octocrab::OctocrabBuilder::new()
        .personal_token(config.personal_token.clone())
        .build()
        .unwrap();

    let opt = Opt::from_args();
    match opt.subcommand {
        Subcommand::Init => println!("config file created if it didn't already exist"),
        Subcommand::Create(create) => {
            subcommand_create(create, &octocrab, &config).await;
        }
        Subcommand::Update(update) => {
            subcommand_update(update, &octocrab, &config).await;
        }
        Subcommand::Delete(delete) => {
            subcommand_delete(delete, &octocrab, &config).await;
        }
    }
}
