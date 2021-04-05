use octocrab::Octocrab;
use rand::prelude::*;
use serde::*;
use std::default::Default;
use structopt::StructOpt;

#[derive(Serialize, Deserialize)]
struct Config {
    personal_token: String,
    base_url: String,
    repo_username: String,
    repo_name: String,
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
    Delete(Delete),
}

#[derive(Debug, StructOpt)]
struct Create {
    #[structopt(long, short)]
    name: Option<String>,
    url: String,
}

#[derive(Debug, StructOpt)]
struct Delete {
    name: String,
}

async fn subcommand_create(create: Create, octocrab: &Octocrab, config: &Config) {
    const IDENT_LEN: usize = 8;

    // TODO: make this const too, if possible
    let alphabet = "abcdefghijklmnopqrstuvwxyz";
    let alphabet_chars = alphabet.chars().collect::<Vec<char>>();

    let Create { name, url } = create;
    let name = name.unwrap_or_else(|| {
        let mut rng = rand::thread_rng();

        (0..IDENT_LEN)
            .map(|_| alphabet_chars.choose(&mut rng).unwrap())
            .collect::<String>()
    });

    let filename = format!("{}/index.html", name);
    let commit_message = format!("Created {} -> {}", name, url);
    let contents = format!(
        r#"<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv="refresh" content="0;url={0}">
        <title>Redirecting...</title>
    </head>
    <body>
        <p><a href="{0}">Click here</a> if you are not redirected</p>
    </body>
</html>"#,
        url
    );

    octocrab
        .repos(&config.repo_username, &config.repo_name)
        .create_file(filename, commit_message, contents)
        .branch("master")
        .send()
        .await
        .unwrap();

    let short_url = format!("{}/{}/", config.base_url.trim_end_matches('/'), name);
    println!(
        "successfully created redirect from {} to {}",
        short_url, url
    );
}

async fn subcommand_delete(delete: Delete, octocrab: &Octocrab, config: &Config) {
    let Delete { name } = delete;

    let file_path = format!("{}/index.html", name);

    #[derive(Serialize)]
    struct GetBody {}

    #[derive(Deserialize)]
    struct GetResponse {
        sha: String,
    }

    let path = format!(
        "/repos/{}/{}/contents/{}",
        config.repo_username, config.repo_name, file_path
    );
    let response = octocrab
        .get::<GetResponse, _, _>(path, Some(&GetBody {}))
        .await
        .unwrap();

    let GetResponse { sha } = response;

    #[derive(Serialize)]
    struct DeleteBody {
        message: String,
        sha: String,
    }

    #[derive(Deserialize)]
    struct DeleteResponse {}

    let path = format!(
        "/repos/{}/{}/contents/{}",
        config.repo_username, config.repo_name, file_path
    );
    let body = DeleteBody {
        message: format!("Delete {}", name),
        sha,
    };
    octocrab
        .delete::<DeleteResponse, _, _>(path, Some(&body))
        .await
        .unwrap();

    println!("successfully removed \"{}\" redirect", name);
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
        Subcommand::Delete(delete) => {
            subcommand_delete(delete, &octocrab, &config).await;
        }
    }
}
