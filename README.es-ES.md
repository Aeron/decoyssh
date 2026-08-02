

# DecoySSH

Es un tarpit SSH compacto y portátil escrito en Rust y `async-std`.

## Motivación

Sí, hay millones de servidores tarpit SSH, además de [el original][original].
Algunos también están escritos en Rust, pero, por lo que he visto, ninguno utiliza `async-std`.
En mi opinión, algunos son un poco excesivos y otros carecen de configurabilidad. Así que aquí está
mi versión.

Sin embargo, este proyecto personal se desarrolló no para competir con nadie, sino para aprender cosas nuevas
y experimentar. No solo con Rust y `async-std`, sino también con las herramientas detrás:
flujos de trabajo de GitHub, compilación cruzada, contenerización, etc. Un ciclo de entrega
bastante completo, en otras palabras. (Pero aún sin pruebas, tal vez algún día.)

A pesar de eso, debería ser 100 % utilizable. Pruébalo si se ajusta a tus necesidades de tarpit.

[original]: https://github.com/skeeto/endlessh

## Uso

DecoySSH está disponible como binarios independientes, un paquete de Cargo y una imagen de contenedor.

Los binarios se pueden encontrar en la [página de versiones][releases] del repositorio. Si no hay una plataforma
que busques, puedes compilar el binario adecuado tú mismo. O siéntete libre de
crear [una PR][pulls] o [un issue][issues].

El paquete de Cargo se puede instalar como de costumbre:

```sh
cargo install decoyssh
```

La imagen de contenedor está disponible como [`docker.io/aeron/decoyssh`][docker] y
[`ghcr.io/Aeron/decoyssh`][github]. Puedes usarlas indistintamente.

```sh
docker pull docker.io/aeron/decoyssh
# …o…
docker pull ghcr.io/aeron/decoyssh
```

[releases]: https://github.com/Aeron/decoyssh/releases
[pulls]: https://github.com/Aeron/decoyssh/pulls
[issues]: https://github.com/Aeron/decoyssh/issues
[docker]: https://hub.docker.com/r/aeron/decoyssh
[github]: https://github.com/Aeron/decoyssh/pkgs/container/decoyssh

### Opciones de la aplicación

Ejecutar la aplicación con la opción `-h` o `--help` te dará lo siguiente:

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

Todas las opciones están disponibles como variables de entorno, con el mismo nombre que los valores
pero con el prefijo `DECOYSSH_`. Por ejemplo, `DECOYSSH_ADDRS`, `DECOYSSH_DELAY`, etc.

> [!NOTE]
> Existen opciones y variables de entorno de compatibilidad con versiones anteriores para direcciones IPv4
> e IPv6 antiguas. Estas tienen los mismos alias que antes: `-4` y `-6`,
> `--ipv4-address` y `--ipv6-address`, `DECOYSSH_IPV4_ADDR` y `DECOYSSH_IPV6_ADDR`
> respectivamente.

### Ejecución en contenedor

Ejecutar un contenedor es bastante sencillo:

```sh
docker -d --restart unless-stopped --name decoyssh \
    --user=65534 \
    -p 22/2222:tcp \
    -e DECOYSSH_PORT=2222 \
    docker.io/aeron/decoyssh
```

De forma predeterminada, la aplicación en contenedor utiliza solo una dirección IPv4 y el puerto `2222` en lugar de
`22`.

Si planeas usar solo enlace IPv4, puedes usar la variable específica del contenedor
`DECOYSSH_PORT` para cambiar el número de puerto de escucha/exposición. De lo contrario, usa
las [variables de entorno](#app-options) estándar explícitamente.

No olvides la técnica del usuario no privilegiado. El contenedor en sí no aplicará
ningún UID específico.
