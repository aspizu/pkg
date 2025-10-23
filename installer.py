#!/usr/bin/env python3
import argparse
import os
import subprocess
from pathlib import Path

import tomli_w

base = """\
configuration iana-etc glibc zlib bzip2 xz lz4 zstd file readline m4 bc flex meowconf
binutils gmp mpfr mpc attr acl libcap libxcrypt shadow gcc ncurses sed psmisc gettext
bison grep bash libtool gdbm gperf expat inetutils less perl perl-xml-parser intltool
autoconf automake openssl libelf libffi python ninja coreutils diffutils gawk findutils
groff gzip iproute2 kbd libpipeline make patch tar texinfo udev procps-ng util-linux
e2fsprogs sysklogd sysvinit
"""


def sh(
    command: str,
    cwd: str | Path | None = None,
    check: bool = True,
    envs: dict[str, str] | None = None,
):
    args = ["bash", "-exc", command]
    env = None
    if envs:
        env = os.environ.copy()
        for key, value in envs.items():
            env[key] = value
    subprocess.run(args, cwd=cwd, check=check, env=env)


def install_base(root: Path):
    sh('sudo install -dm755 "$DESTDIR"/tmp/meow')
    sh('sudo install -dm755 "$DESTDIR"/var/lib/meow')
    sh('sudo install -dm755 "$DESTDIR"/etc/meow')
    config = tomli_w.dumps(
        {
            "index": "https://tilde.club/~aspizu/",
            "packages": [meow.strip() for meow in base.split() if meow.strip()],
        }
    )
    sh(
        'echo "$meowCONFIG" | sudo tee "$DESTDIR"/etc/meow/config.toml',
        envs={"meowCONFIG": config},
    )
    sh('sudo target/release/meow --root "$DESTDIR" sync')
    sh('sudo rm -rf "$DESTDIR"/tmp')


def install_meow(root: Path):
    sh('sudo install -Dm755 target/release/meow "$DESTDIR"/usr/bin/meow')


argparser = argparse.ArgumentParser()
argparser.add_argument("root", help="The root filesystem to install to.")
argparser.add_argument(
    "--rm", help="Remove existing installation.", action="store_true"
)
args = argparser.parse_args()
root = Path(args.root).absolute().resolve()
if root.as_posix() in ["/", ".", "/usr", "/bin", "/etc"]:
    raise RuntimeError("Refusing to install to host root")
os.environ["DESTDIR"] = root.as_posix()
if args.rm:
    sh('sudo rm -rf "$DESTDIR"/*', check=False)
install_base(root)
install_meow(root)
