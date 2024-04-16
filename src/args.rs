use std::env;

use async_std::net::SocketAddr;
use clap::error::{ContextKind, ContextValue, DefaultFormatter, Error, ErrorKind};
use clap::{crate_description, crate_name, crate_version, value_parser, CommandFactory, Parser};

#[derive(Parser, Debug)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    about = crate_description!(),
)]
pub struct Args {
    #[clap(
        short,
        long = "capacity",
        value_parser = value_parser!(u16).range(1..),
        env = "DECOYSSH_CAP",
        hide_env(true),
        default_value = "4096",
        display_order(5),
        help = "Maximum number of connections",
    )]
    pub cap: u16,

    #[clap(
        short,
        long,
        value_parser = value_parser!(u64).range(1..),
        env = "DECOYSSH_DELAY",
        hide_env(true),
        default_value = "10000",
        display_order(3),
        help = "Message delay (in milliseconds)"
    )]
    pub delay: u64,

    #[clap(
        short,
        long,
        value_parser = value_parser!(u8).range(3..),
        env = "DECOYSSH_LENGTH",
        hide_env(true),
        default_value = "32",
        display_order(4),
        help = "Maximum line length"
    )]
    pub length: u8,

    #[clap(
        short,
        long = "address",
        env = "DECOYSSH_ADDRS",
        hide_env(true),
        num_args(0..),
        default_value = "0.0.0.0:22",
        display_order(1),
        help = "IP address(es) to bind on",
        // NOTE: backward compatibility
        short_aliases = ['4', '6'],
        aliases = ["ipv4-address", "ipv6-address"],
    )]
    pub addrs: Vec<SocketAddr>,
}

impl Args {
    const ADDR_ENV_ALIASES: [&'static str; 2] = ["DECOYSSH_IPV4_ADDR", "DECOYSSH_IPV6_ADDR"];

    fn exit_with_env_value_validation_error(key: &str, value: &str) -> ! {
        let cmd = <Self as CommandFactory>::command();
        let mut error: Error<DefaultFormatter> =
            Error::new(ErrorKind::ValueValidation).with_cmd(&cmd);

        error.insert(
            ContextKind::InvalidArg,
            ContextValue::String(key.to_owned()),
        );
        error.insert(
            ContextKind::InvalidValue,
            ContextValue::String(value.to_owned()),
        );

        error.exit()
    }

    fn enrich_with_compatible_env_addrs(addrs: &mut Vec<SocketAddr>) {
        for key in Self::ADDR_ENV_ALIASES {
            let Ok(value) = env::var(key) else { continue };

            for value in value.split(',').map(|value| value.trim()) {
                match value.parse() {
                    Ok(addr) => addrs.push(addr),
                    Err(_) => Self::exit_with_env_value_validation_error(key, value),
                }
            }
        }

        if addrs.len() > 1 {
            // NOTE: assuming the default value is now excessive
            addrs.remove(0);
            addrs.dedup();
        }
    }

    pub fn parse() -> Args {
        let mut args = <Self as Parser>::parse();
        args.addrs.dedup();

        // HACK: keeping backward compatibility with older IPv4 and IPv6 keys
        // since Clap has no environment variable name aliases at the moment
        // (see https://github.com/clap-rs/clap/issues/5447)
        if args.addrs.len() == 1 && args.addrs[0].to_string() == "0.0.0.0:22" {
            Self::enrich_with_compatible_env_addrs(&mut args.addrs);
        }

        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_setup<S, D, T>(setup: S, teardown: D, test: T)
    where
        T: FnOnce() + std::panic::UnwindSafe,
        S: FnOnce(),
        D: FnOnce(),
    {
        setup();
        let result = std::panic::catch_unwind(test);
        teardown();
        assert!(result.is_ok())
    }

    #[test]
    fn test_enrich_with_compatible_env_addrs() {
        with_setup(
            || {
                env::set_var("DECOYSSH_IPV4_ADDR", "127.0.0.1:22,127.0.0.2:22");
                env::set_var("DECOYSSH_IPV6_ADDR", "[::ffff:7f00:3]:22");
            },
            || {
                for key in Args::ADDR_ENV_ALIASES {
                    env::remove_var(key);
                }
            },
            || {
                let mut addrs: Vec<SocketAddr> = vec!["0.0.0.0:22".parse().unwrap()];

                Args::enrich_with_compatible_env_addrs(&mut addrs);

                assert!(addrs.len() == 3, "{addrs:?}");
                assert!(addrs[0] == "127.0.0.1:22".parse().unwrap());
                assert!(addrs[1] == "127.0.0.2:22".parse().unwrap());
                assert!(addrs[2] == "[::ffff:7f00:3]:22".parse().unwrap());
            },
        )
    }
}
