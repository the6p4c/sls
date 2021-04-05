sls
===
~~shitty~~ static link shortener

**why?**  
i don't want a vps

**how**  
makes a folder for each link name, puts an `index.html` in there that does a meta redirect

**how (2)**  
`sls init` will make a config in like `~/.config/sls/sls.toml` if it doesn't exist. fill it out with your details

- `personal_token` should be a github personal token from [here](https://github.com/settings/tokens)
- `base_url` should be where the github pages site is hosted, like `https://[username].github.io/[repo_name]/` for a repo hosted the default way or `https://[subdomain].[your_domain].[tld]/` if you're doing CNAME stuff
- `repo_username` and `repo_name` are the username and name of the repo that github pages is hosting for you

`sls create [link]` makes a short link to a link. specify `-n name` if you want to give the short link a nice name

`sls delete [name]` deletes a short link with the specified name
