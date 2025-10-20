#!/usr/bin/env bash
set -ex

if [[ -z "$1" ]]; then
    echo "Usage: ./pkgstrap.sh /mnt"
fi

if [[ ! -d "$1" ]]; then
    echo "$1 is not a directory"
fi

export LFS="$1"
umask 022

# 4.2 --- FILESYSTEM HIERARCHY STANDARD ---

mkdir -pv "$LFS"/{etc,var} "$LFS"/usr/{bin,lib,sbin}

for i in bin lib sbin; do
  ln -sv usr/$i "$LFS"/$i
done

case $(uname -m) in
  x86_64) mkdir -pv "$LFS"/lib64 ;;
esac

mkdir -pv "$LFS"/tools

# These steps can be done earlier:

# 7.2 Virtual kernel file systems
mkdir -pv "$LFS"/{dev,proc,sys,run}

# 7.5 creating directories

mkdir -pv "$LFS"/{boot,home,mnt,opt,srv}

mkdir -pv "$LFS"/etc/{opt,sysconfig}
mkdir -pv "$LFS"/lib/firmware
mkdir -pv "$LFS"/media/{floppy,cdrom}
mkdir -pv "$LFS"/usr/{,local/}{include,src}
mkdir -pv "$LFS"/usr/lib/locale
mkdir -pv "$LFS"/usr/local/{bin,lib,sbin}
mkdir -pv "$LFS"/usr/{,local/}share/{color,dict,doc,info,locale,man}
mkdir -pv "$LFS"/usr/{,local/}share/{misc,terminfo,zoneinfo}
mkdir -pv "$LFS"/usr/{,local/}share/man/man{1..8}
mkdir -pv "$LFS"/var/{cache,local,log,mail,opt,spool}
mkdir -pv "$LFS"/var/lib/{color,misc,locate}

ln -sfv /run "$LFS"/var/run
ln -sfv /run/lock "$LFS"/var/lock

install -dv -m 0750 "$LFS"/root
install -dv -m 1777 "$LFS"/tmp "$LFS"/var/tmp

# --- 9.6. System V Bootscript Usage and Configuration ---

cat > $LFS/etc/inittab << "EOF"
# Begin /etc/inittab

id:3:initdefault:

si::sysinit:/etc/rc.d/init.d/rc S

l0:0:wait:/etc/rc.d/init.d/rc 0
l1:S1:wait:/etc/rc.d/init.d/rc 1
l2:2:wait:/etc/rc.d/init.d/rc 2
l3:3:wait:/etc/rc.d/init.d/rc 3
l4:4:wait:/etc/rc.d/init.d/rc 4
l5:5:wait:/etc/rc.d/init.d/rc 5
l6:6:wait:/etc/rc.d/init.d/rc 6

ca:12345:ctrlaltdel:/sbin/shutdown -t1 -a -r now

su:S06:once:/sbin/sulogin
s1:1:respawn:/sbin/sulogin

1:2345:respawn:/sbin/agetty --noclear tty1 9600
2:2345:respawn:/sbin/agetty tty2 9600
3:2345:respawn:/sbin/agetty tty3 9600
4:2345:respawn:/sbin/agetty tty4 9600
5:2345:respawn:/sbin/agetty tty5 9600
6:2345:respawn:/sbin/agetty tty6 9600

# End /etc/inittab
EOF

cat > $LFS/etc/sysconfig/clock << "EOF"
# Begin /etc/sysconfig/clock

UTC=1

# Set this to any options you might need to give to hwclock,
# such as machine hardware clock type for Alphas.
CLOCKPARAMS=

# End /etc/sysconfig/clock
EOF

cat > $LFS/etc/sysconfig/console << "EOF"
# Begin /etc/sysconfig/console

UNICODE="1"
FONT="Lat2-Terminus16"

# End /etc/sysconfig/console
EOF

# --- 9.7. Configuring the System Locale ---

cat > $LFS/etc/profile << "EOF"
# Begin /etc/profile

for i in $(locale); do
  unset ${i%=*}
done

if [[ "$TERM" = linux ]]; then
  export LANG=C.UTF-8
else
  export LANG=en_US.UTF-8
fi

# End /etc/profile
EOF

# --- 9.8. Creating the /etc/inputrc File ---

cat > $LFS/etc/inputrc << "EOF"
# Begin /etc/inputrc
# Modified by Chris Lynn <roryo@roryo.dynup.net>

# Allow the command prompt to wrap to the next line
set horizontal-scroll-mode Off

# Enable 8-bit input
set meta-flag On
set input-meta On

# Turns off 8th bit stripping
set convert-meta Off

# Keep the 8th bit for display
set output-meta On

# none, visible or audible
set bell-style none

# All of the following map the escape sequence of the value
# contained in the 1st argument to the readline specific functions
"\eOd": backward-word
"\eOc": forward-word

# for linux console
"\e[1~": beginning-of-line
"\e[4~": end-of-line
"\e[5~": beginning-of-history
"\e[6~": end-of-history
"\e[3~": delete-char
"\e[2~": quoted-insert

# for xterm
"\eOH": beginning-of-line
"\eOF": end-of-line

# for Konsole
"\e[H": beginning-of-line
"\e[F": end-of-line

# End /etc/inputrc
EOF

# --- 9.9. Creating the /etc/shells File ---

cat > $LFS/etc/shells << "EOF"
# Begin /etc/shells

/bin/sh
/bin/bash

# End /etc/shells
EOF

# --- 10.2. Creating the /etc/fstab File ---

cat > $LFS/etc/fstab << "EOF"
# Begin /etc/fstab

# file system  mount-point    type     options             dump  fsck
#                                                                order

/dev/<xxx>     /              <fff>    defaults            1     1
/dev/<yyy>     swap           swap     pri=1               0     0
proc           /proc          proc     nosuid,noexec,nodev 0     0
sysfs          /sys           sysfs    nosuid,noexec,nodev 0     0
devpts         /dev/pts       devpts   gid=5,mode=620      0     0
tmpfs          /run           tmpfs    defaults            0     0
devtmpfs       /dev           devtmpfs mode=0755,nosuid    0     0
tmpfs          /dev/shm       tmpfs    nosuid,nodev        0     0
cgroup2        /sys/fs/cgroup cgroup2  nosuid,noexec,nodev 0     0

# End /etc/fstab
EOF

cargo install --path . --root $LFS/usr
mkdir -p /var/lib/pkg
mkdir -p /etc/pkg
cat > $LFS/etc/pkg/config.toml << "EOF"
index = "https://tilde.club/~aspizu/"
packages = [
    "acl",
    "attr",
    "autoconf",
    "automake",
    "bash",
    "bc",
    "binutils",
    "bison",
    "bzip2",
    "coreutils",
    "diffutils",
    "e2fsprogs",
    "expat",
    "file",
    "findutils",
    "flex",
    "gawk",
    "gcc",
    "gdbm",
    "gettext",
    "glibc",
    "gmp",
    "gperf",
    "grep",
    "groff",
    "gzip",
    "iana-etc",
    "inetutils",
    "intltool",
    "iproute2",
    "kbd",
    "less",
    "libcap",
    "libelf",
    "libffi",
    "libpipeline",
    "libtool",
    "libxcrypt",
    "lz4",
    "m4",
    "make",
    "mpc",
    "mpfr",
    "ncurses",
    "ninja",
    "openssl",
    "patch",
    "perl",
    "perl-xml-parser",
    "pkgconf",
    "procps-ng",
    "psmisc",
    "python",
    "readline",
    "sed",
    "shadow",
    "sysklogd",
    "sysvinit",
    "tar",
    "texinfo",
    "udev",
    "udev-lfs",
    "util-linux",
    "xz",
    "zlib",
    "zstd",
]
EOF
$LFS/usr/bin/pkg --root $LFS sync
rm -rf $LFS/tmp
