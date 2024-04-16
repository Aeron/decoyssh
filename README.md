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

The container image is available as [`docker.io/aeron/decoyssh`][docker] and
[`ghcr.io/Aeron/decoyssh`][github]. You can use them both interchangeably.

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
Usage: decoyssh [OPTIONS]

Options:
  -a, --address [<ADDRS>...]  IP address(es) to bind on [default: 0.0.0.0:22]
  -d, --delay <DELAY>         Message delay (in milliseconds) [default: 10000]
  -l, --length <LENGTH>       Maximum line length [default: 32]
  -c, --capacity <CAP>        Maximum number of connections [default: 4096]
  -h, --help                  Print help
  -V, --version               Print version
```

All options are available as environment variables, with the same name as value names
but with the `DECOYSSH_` prefix. For example, `DECOYSSH_ADDRS`, `DECOYSSH_DELAY`, etc.

> [!NOTE]
> There are backward compatibility options and environment variables for older IPv4
> and IPv6 addresses available. Those have the same aliases as before: `-4` and `-6`,
> `--ipv4-address` and `--ipv6-address`, `DECOYSSH_IPV4_ADDR` and `DECOYSSH_IPV6_ADDR`
> respectively.

### Container Running

Running a container is pretty straigthforward:

```sh
docker -d --restart unless-stopped --name decoyssh \
    --user=65534 \
    -p 22/2222:tcp \
    -e DECOYSSH_PORT=2222 \
    docker.io/aeron/decoyssh
```

By default, the containerized app uses only an IPv4 address and `2222` port instead of
`22`.

If you’re planning to use IPv4 binding only, you can use the container-specific
`DECOYSSH_PORT` variable to change the listening/exposed port number. Otherwise, use
standard [environment variables](#app-options) explicitly.

Don’t forget about the unprivileged user trick. The container itself won’t enforce
any specific UID.
