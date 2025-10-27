#!/usr/bin/env python3
import argparse
import os
import subprocess
from pathlib import Path

import tomli_w

base = """\
configuration acl        flex     groff            libcap         m4       patch            sysklogd    zlib
attr          coreutils  gawk     gzip             libelf         make     perl             sysvinit    zstd
autoconf      curl       gcc      iana-etc         libffi         make-ca  perl-xml-parser  tar
automake      dash       gdbm     inetutils        libpipeline    mpc      pkgconf          texinfo
bash          diffutils  gettext  intltool         libpsl         mpfr     procps-ng        tzdata
bc            e2fsprogs  git      iproute2         libtasn1       ncurses  psmisc           udev
binutils      expat      glibc    kakoune          libtool        ninja    python           udev-lfs
bison         fastfetch  gmp      kbd              libxcrypt      openssh  readline         util-linux
bzip2         file       gperf    less             linux-headers  openssl  sed              wget
cmake         findutils  grep     lfs-bootscripts  lz4            p11-kit  shadow           xz
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
            "keys": ["RWR5G7Ii33vLdX3oxrrc7+8QhVivVZmtMrJU/JsFRmFXZBAVVBR70Ilr"],
        }
    )
    sh(
        'echo "$meowCONFIG" | sudo tee "$DESTDIR"/etc/meow/meow.toml',
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
