#!/usr/bin/env bash
set -euf -o pipefail

TARGET_ARCH=$(uname -m | sed 's/aarch64/arm64/' | sed 's/x86_64/amd64/')

apt-get update

# Install required packages
#
# The FDB version should match `cluster::workflows::server::install::install_scripts::components::fdb::FDB_VERSION`
 apt-get install -y \
	libclang-dev \
    ca-certificates \
    openssl \
    curl \
    postgresql-client \
    gpg \
    xz-utils \
	unzip \
    apt-transport-https \
    dirmngr \
	netcat-openbsd \
	procps 

(curl -L https://github.com/golang-migrate/migrate/releases/download/v4.18.1/migrate.linux-${TARGET_ARCH}.tar.gz | tar xvz)
mv migrate /usr/local/bin/migrate

curl -fsSL https://deno.land/x/install/install.sh | sh
ln -s /root/.deno/bin/deno /usr/local/bin/deno

curl -Lf -o /lib/libfdb_c.so "https://github.com/apple/foundationdb/releases/download/7.1.60/libfdb_c.x86_64.so"

# === Traefik ===
curl -sSLf https://github.com/traefik/traefik/releases/download/v${TRAEFIK_VERSION}/traefik_v${TRAEFIK_VERSION}_linux_${TARGET_ARCH}.tar.gz | \
    tar xz -C /usr/local/bin/ traefik

# === CockroachDB ===
useradd -m -s /bin/bash cockroachdb && \
    curl -sSLf https://binaries.cockroachdb.com/cockroach-v${COCKROACHDB_VERSION}.linux-${TARGET_ARCH}.tgz | tar xz && \
    cp -i cockroach-v${COCKROACHDB_VERSION}.linux-${TARGET_ARCH}/cockroach /usr/local/bin/ && \
    rm -rf cockroach-v${COCKROACHDB_VERSION}.linux-${TARGET_ARCH}

# === Redis ===
# TODO(RVT-4084): Switch to Valkey when Debian 13 released or ocmpile from source
useradd -m -s /bin/bash redis && \
    apt install -y redis-server redis-tools

# === ClickHouse ===
useradd -m -s /bin/bash clickhouse && \
    curl -fsSL 'https://packages.clickhouse.com/rpm/lts/repodata/repomd.xml.key' | gpg --dearmor -o /usr/share/keyrings/clickhouse-keyring.gpg && \
    echo "deb [signed-by=/usr/share/keyrings/clickhouse-keyring.gpg] https://packages.clickhouse.com/deb stable main" | tee /etc/apt/sources.list.d/clickhouse.list && \
    apt-get update && \
    apt-get install -y clickhouse-client clickhouse-server

# === NATS ===
useradd -m -s /bin/bash nats && \
    curl -sSLf https://github.com/nats-io/nats-server/releases/download/v${NATS_VERSION}/nats-server-v${NATS_VERSION}-linux-${TARGET_ARCH}.tar.gz | \
    tar xz -C /usr/local/bin/ --strip-components=1 nats-server-v${NATS_VERSION}-linux-${TARGET_ARCH}/nats-server

# === SeaweedFS ===
useradd -m -s /bin/bash seaweedfs && \
    curl -sSLf https://github.com/seaweedfs/seaweedfs/releases/download/${SEAWEEDFS_VERSION}/linux_${TARGET_ARCH}.tar.gz | tar xz -C /usr/local/bin/

# === FoundationDB ===
# Client (for health checks)
curl -sSLf -o "/tmp/foundationdb-clients.deb" "https://github.com/apple/foundationdb/releases/download/${FDB_VERSION}/foundationdb-clients_${FDB_VERSION}-1_amd64.deb"
dpkg -i "/tmp/foundationdb-clients.deb"

fdbcli --version

# Server
curl -Lf -o "/tmp/foundationdb-server.deb" "https://github.com/apple/foundationdb/releases/download/${FDB_VERSION}/foundationdb-server_${FDB_VERSION}-1_amd64.deb"
dpkg -i "/tmp/foundationdb-server.deb"
rm -rf /etc/foundationdb

fdbserver --version

# Create log dir for internal FDB logs, since /var/log/foundationdb is used by S6
mkdir /var/log/foundationdb-monitor
chown foundationdb:foundationdb /var/log/foundationdb-monitor

# === Vector ===
useradd -m -s /bin/bash vector-client && \
	useradd -m -s /bin/bash vector-server && \
    curl -sSLf https://packages.timber.io/vector/${VECTOR_VERSION}/vector_${VECTOR_VERSION}-1_${TARGET_ARCH}.deb -o /tmp/vector.deb && \
    dpkg -i /tmp/vector.deb && \
    rm /tmp/vector.deb

# === S6 Overlay ===
curl -sSLf https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz -o /tmp/s6-overlay-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    curl -sSLf https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-$(uname -m).tar.xz -o /tmp/s6-overlay-$(uname -m).tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-$(uname -m).tar.xz

# Setup S6
deno run --allow-read --allow-write /tmp/build-scripts/setup_s6.ts

# === Rivet ===
useradd -m -s /bin/bash rivet-server
useradd -m -s /bin/bash rivet-guard
