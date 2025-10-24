#!/usr/bin/env bash
LFS="$1"

if [[ -z "$LFS" || "$LFS" = "/" ]]; then
    echo "Refusing to chroot to /"
    exit 1
fi

mount --bind /dev $LFS/dev
mount -t devpts devpts -o gid=5,mode=0620 $LFS/dev/pts
mount -t proc proc $LFS/proc
mount -t sysfs sysfs $LFS/sys
mount -t tmpfs tmpfs $LFS/run
if [ -h $LFS/dev/shm ]; then
  install -d -m 1777 "$LFS$(realpath /dev/shm)"
else
  mount -t tmpfs -o nosuid,nodev tmpfs $LFS/dev/shm
fi
chroot "$LFS" /usr/bin/env -i   \
    HOME=/root                  \
    TERM="$TERM"                \
    PS1='(lfs-chroot) \[\033[01;32m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]\$ ' \
    PATH=/usr/bin:/usr/sbin     \
    MAKEFLAGS="-j$(nproc)"      \
    TESTSUITEFLAGS="-j$(nproc)" \
    /bin/bash --login
umount -R $LFS/dev/shm
umount -R $LFS/dev/pts
umount -R $LFS/{sys,proc,run,dev}
