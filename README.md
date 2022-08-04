# DecoySSH

It’s a compact and portable SSH tarpit written in Rust and `async-std`.

## Motivation

Yup, there are millions of SSH tarpit servers, besides [the original one][original].
Some are written in Rust as well, but—as far as I’ve seen—none of them use `async-std`.
To my taste, some of them are a bit too much, and some lack configurability. So here’s
my take.

Yet this pet project developed not to compete with anyone but to learn new things
and experiment. Not just with Rust and `async-std` but also with things behind:
GitHub workflows, cross-compiling, containerization, etc. A somewhat complete
delivery cycle, in other words. (But no tests yet, maybe someday.)

Despite that, it should be 100% usable. Give it a try if it suits your tarpit needs.

[original]: https://github.com/skeeto/endlessh

## Usage

DecoySSH is available as stand-alone binaries, a Cargo package, and a container image.

Binaries can be found on the repo’s [releases page][releases]. If there’s no platform
you’re looking for, you can compile an appropriate binary yourself. Or feel free to
create [a PR][pulls] or [an issue][issues].

Cargo package can be installed as usually:

```sh
cargo install decoyssh
```

The container image is available as [`docker.io/aeron/decoyssh`][docker] from Docker
Hub and [`ghcr.io/Aeron/decoyssh`][github] from GitHub Container Registry. You can use
them both interchangeably.

```sh
docker pull docker.io/aeron/decoyssh
# …or…
docker pull ghcr.io/aeron/decoyssh
```

[releases]: https://github.com/Aeron/decoyssh/releases
[pulls]: https://github.com/Aeron/decoyssh/pulls
[issues]: https://github.com/Aeron/decoyssh/issues
[docker]: https://hub.docker.com/r/aeron/decoyssh
[github]: https://github.com/Aeron/decoyssh/pkgs/container/decoyssh

### App Options

Running the app with `-h` or `--help` option will give you the following:

```text
USAGE:
    decoyssh [OPTIONS]

OPTIONS:
    -4, --ipv4-address <IPV4_ADDR>...
            IPv4 address(es) to bind on [max: 8]

    -6, --ipv6-address <IPV6_ADDR>...
            IPv6 address(es) to bind on [max: 8]

    -d, --delay <DELAY>
            Message delay (in milliseconds) [default: 10000]

    -l, --length <LENGTH>
            Maximum line length [default: 32]

    -c, --capacity <CAP>
            Maximum number of connections [default: 4096]

    -h, --help
            Print help information

    -V, --version
            Print version information
```

If no addresses are given, it’ll run on `0.0.0.0:22` only. To use both IPv4 and
IPv6 addresses, both options—with or without values—must be given explicitly.

All options are available as environment variables, with the same name as value names
but with the `DECOYSSH_` prefix. For example, `DECOYSSH_IPV4_ADDR`.

### Container Running

Running a container is pretty straigthforward:

```sh
docker -d --restart unless-stopped --name decoyssh \
    -p 22/2222:tcp \
    -e DECOYSSH_PORT=2222 \
    docker.io/aeron/decoyssh
```

The containerized app utilizes `2222` port by default instead of `22`. If you’re
planning to use IPv4 binding only, you can use the container-specific `DECOYSSH_PORT`
variable to change the listening/exposed port number. In the case of IPv6 or both
addresses, use standard [environment variables](#app-options).
