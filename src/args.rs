use async_std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use clap::{value_parser, Parser};
use shadow_rs::shadow;

shadow!(build);

#[derive(Parser, Debug)]
#[clap(name = build::PROJECT_NAME)]
#[clap(
    version = build::PKG_VERSION,
    long_version = build::CLAP_LONG_VERSION,
    about = build::PKG_DESCRIPTION,
    long_about = None,
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
        value_parser = value_parser!(u8).range(3..=255),
        env = "DECOYSSH_LENGTH",
        hide_env(true),
        default_value = "32",
        display_order(4),
        help = "Maximum line length"
    )]
    pub length: u8,

    #[clap(
        short = '4',
        long = "ipv4-address",
        env = "DECOYSSH_IPV4_ADDR",
        hide_env(true),
        num_args(0..=8),
        default_missing_value = "0.0.0.0:22",
        display_order(1),
        help = "IPv4 address(es) to bind on [max: 8]"
    )]
    pub ipv4_addr: Option<Vec<SocketAddrV4>>,

    #[clap(
        short = '6',
        long = "ipv6-address",
        env = "DECOYSSH_IPV6_ADDR",
        hide_env(true),
        num_args(0..=8),
        default_missing_value = "[::]:22",
        display_order(2),
        help = "IPv6 address(es) to bind on [max: 8]"
    )]
    pub ipv6_addr: Option<Vec<SocketAddrV6>>,
}

impl Args {
    const DEFAULT_IP: Ipv4Addr = Ipv4Addr::new(0, 0, 0, 0);
    const DEFAULT_PORT: u16 = 22;

    fn default_addr() -> SocketAddrV4 {
        SocketAddrV4::new(Self::DEFAULT_IP, Self::DEFAULT_PORT)
    }

    pub fn addrs(&self) -> Vec<SocketAddr> {
        let mut addrs: Vec<SocketAddr> = Vec::with_capacity(16);

        if let Some(addr) = &self.ipv4_addr {
            addrs.extend(addr.iter().map(|a| SocketAddr::V4(*a)));
        }

        if let Some(addr) = &self.ipv6_addr {
            addrs.extend(addr.iter().map(|a| SocketAddr::V6(*a)));
        }

        addrs.dedup();

        addrs
    }

    // HACK: The default_value_if works only with a value of a present argument, not
    // with the presence of one itself. So it seems impossible to have a default value
    // for an argument solely in the absence of another one. The following is a
    // workaround.
    // TODO: check if it is still the case in Clap v4
    pub fn parse() -> Args {
        let mut args = <Self as Parser>::parse();

        if args.ipv4_addr.is_none() && args.ipv6_addr.is_none() {
            args.ipv4_addr = Some([Self::default_addr()].to_vec());
        }

        args
    }
}
