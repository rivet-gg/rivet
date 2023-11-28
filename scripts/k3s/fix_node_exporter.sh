#!/bin/sh
set -euf

# This needs to be run after a system reboot if you are using k3d in k3d as the
# development environment. See more details here:
# https://github.com/rivet-gg/rivet/issues/208

docker exec k3d-rivet-dev-server-0 \
    mount --make-rshared /
