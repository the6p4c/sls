use crate::Config;
use octocrab::Octocrab;
use serde::*;

fn path_from_name(name: &str) -> String {
    format!("{}/index.html", name)
}

#[derive(Debug)]
pub struct Shortlink<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub sha: Option<&'a str>,
}

impl Shortlink<'_> {
    pub async fn send(self, octocrab: &Octocrab, config: &Config) -> String {
        let path = path_from_name(self.name);

        let verb = if self.sha.is_some() {
            "Updated"
        } else {
            "Created"
        };
        let message = format!("{} {} -> {}", verb, self.name, self.url);

        let content = format!(
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
            self.url
        );

        match self.sha {
            Some(sha) => {
                octocrab
                    .repos(&config.repo_username, &config.repo_name)
                    .update_file(path, message, content, sha)
                    .branch("master")
                    .send()
                    .await
                    .unwrap();
            }
            None => {
                octocrab
                    .repos(&config.repo_username, &config.repo_name)
                    .create_file(path, message, content)
                    .branch("master")
                    .send()
                    .await
                    .unwrap();
            }
        }

        let shortlink_url = format!("{}/{}/", config.base_url.trim_end_matches('/'), self.name);

        shortlink_url
    }
}

pub async fn get_sha(name: &str, octocrab: &Octocrab, config: &Config) -> String {
    #[derive(Serialize)]
    struct Body {}

    #[derive(Deserialize)]
    struct Response {
        sha: String,
    }

    let path = format!(
        "/repos/{}/{}/contents/{}",
        config.repo_username,
        config.repo_name,
        path_from_name(name)
    );
    let response = octocrab
        .get::<Response, _, _>(path, Some(&Body {}))
        .await
        .unwrap();

    response.sha
}

pub async fn delete(name: &str, sha: String, octocrab: &Octocrab, config: &Config) {
    #[derive(Serialize)]
    struct Body {
        message: String,
        sha: String,
    }

    #[derive(Deserialize)]
    struct Response {}

    let path = format!(
        "/repos/{}/{}/contents/{}",
        config.repo_username,
        config.repo_name,
        path_from_name(name)
    );
    let body = Body {
        message: format!("Delete {}", name),
        sha,
    };
    octocrab
        .delete::<Response, _, _>(path, Some(&body))
        .await
        .unwrap();
}
