use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author = "Sung Noh",
    version = "1.0.0",
    about = "Parlance stored in a database."
)]
pub struct Cli {
    #[arg(value_name = "PHRASE")]
    pub phrase: Option<String>,

    #[arg(short, long, value_name = "EXAMPLE")]
    pub example: Option<String>,

    #[arg(short, long, conflicts_with = "destroy")]
    pub list: bool,

    #[arg(short, long, conflicts_with = "list")]
    pub destroy: Option<String>,
}

impl PartialEq for Cli {
    fn eq(&self, other: &Cli) -> bool {
        self.phrase == other.phrase && self.example == other.example && self.list == other.list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_cli_options() {
        let args = vec!["", "hints", "-e", "Give the compiler some hints."];
        let result = Cli::parse_from(args);

        let expected = Cli {
            phrase: Some("hints".to_string()),
            example: Some("Give the compiler some hints.".to_string()),
            list: false,
            destroy: None,
        };

        assert_eq!(result, expected);

        let args = vec!["", "-l"];
        let result = Cli::parse_from(args);

        let expected = Cli {
            phrase: None,
            example: None,
            list: true,
            destroy: None,
        };

        assert_eq!(result, expected);

        let args = vec!["", "-d", "hello"];
        let result = Cli::parse_from(args);

        let expected = Cli {
            phrase: None,
            example: None,
            list: false,
            destroy: Some("hello".to_string()),
        };

        assert_eq!(result, expected);
    }
}
