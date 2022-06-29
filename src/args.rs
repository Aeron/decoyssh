use clap::{App, Arg, ArgMatches};
use shadow_rs::shadow;

shadow!(build);

pub struct Args {
    pub port: u16,
    pub cap: u16,
    pub delay: u64,
    pub length: u8,
}

impl Args {
    pub fn ipv4_addr(&self) -> String {
        format!("0.0.0.0:{}", self.port)
    }
    pub fn ipv6_addr(&self) -> String {
        format!("[::]:{}", self.port)
    }
}

impl From<ArgMatches> for Args {
    fn from(args: ArgMatches) -> Self {
        Self {
            port: *args.get_one("port").unwrap(),
            cap: *args.get_one("cap").unwrap(),
            delay: *args.get_one("delay").unwrap(),
            length: *args.get_one("length").unwrap(),
        }
    }
}

pub fn parse_app_args() -> Args {
    let args = App::new(build::PROJECT_NAME)
        .version(build::PKG_VERSION)
        .long_version(build::CLAP_LONG_VERSION)
        .about(build::PKG_DESCRIPTION)
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .env("DECOYSSH_PORT")
                .hide_env(true)
                .takes_value(true)
                .value_parser(clap::value_parser!(u16).range(1..))
                .default_value("22")
                .help("Port to listen on"),
        )
        .arg(
            Arg::new("delay")
                .short('d')
                .long("delay")
                .env("DECOYSSH_DELAY")
                .hide_env(true)
                .takes_value(true)
                .value_parser(clap::value_parser!(u64).range(1..))
                .default_value("10000")
                .help("Message delay (in milliseconds)"),
        )
        .arg(
            Arg::new("length")
                .short('l')
                .long("length")
                .env("DECOYSSH_LENGTH")
                .hide_env(true)
                .takes_value(true)
                .value_parser(clap::value_parser!(u8).range(3..=255))
                .default_value("32")
                .help("Maximum line length"),
        )
        .arg(
            Arg::new("cap")
                .short('c')
                .long("capacity")
                .env("DECOYSSH_CAP")
                .hide_env(true)
                .takes_value(true)
                .value_parser(clap::value_parser!(u16).range(1..))
                .default_value("4096")
                .help("Maximum number of connections"),
        )
        // .arg(
        //     Arg::new("ipv4")
        //         .short('4')
        //         .long("ipv4-only")
        //         .env("DECOYSSH_IPV4_ONLY")
        //         .hide_env(true)
        //         .takes_value(false)
        //         .value_parser(clap::value_parser!(bool))
        //         .default_value("false")
        //         .conflicts_with("ipv6")
        //         .help("Bind to IPv4 only"),
        // )
        // .arg(
        //     Arg::new("ipv6")
        //         .short('6')
        //         .long("ipv6-only")
        //         .env("DECOYSSH_IPV6_ONLY")
        //         .hide_env(true)
        //         .takes_value(false)
        //         .value_parser(clap::value_parser!(bool))
        //         .default_value("false")
        //         .conflicts_with("ipv4")
        //         .help("Bind to IPv6 only"),
        // )
        .get_matches();

    Args::from(args)
}
