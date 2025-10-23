#!/usr/bin/env bash
cat << EOF | podman run --rm debian:13 /usr/bin/bash
apt update
apt upgrade -y
EOF
