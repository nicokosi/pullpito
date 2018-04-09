use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    let token_info = if config.token.is_some() { "with token" } else { "without token" };
    println!("Computing stats for GitHub repo {} {}" , config.repo, token_info);
}

#[derive(Debug, PartialEq)]
struct Config {
    repo: String,
    token: Option<String>
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("Not enough arguments, expecting at least 1 argument");
        }
        let repo = args[1].clone();
        let token = if args.len() == 3 {
            Some(args[2].clone())
        } else {
            None
        };
        Ok(Config { repo, token })
    }
}

#[cfg(test)]
mod test {

    use Config;

    #[test]
    fn parse_config_with_no_params() {
        let args: Vec<String> = vec!["".to_string()];
        assert_eq!(Config::new(&args), Err("Not enough arguments, expecting at least 1 argument"));
    }

    #[test]
    fn parse_config_with_repo_param() {
        let args: Vec<String> = vec!["".to_string(), "fakeRepo".to_string()];
        let repo = "fakeRepo".to_string();
        let token = None;
        assert_eq!(Config::new(&args), Ok(Config { repo, token }));
    }

    #[test]
    fn parse_config_with_repo_and_token_params() {
        let args: Vec<String> = vec!["".to_string(), "fakeRepo".to_string(), "fakeToken".to_string()];
        let repo = "fakeRepo".to_string();
        let token = Some("fakeToken".to_string());
        assert_eq!(Config::new(&args), Ok(Config { repo, token }));
    }
}