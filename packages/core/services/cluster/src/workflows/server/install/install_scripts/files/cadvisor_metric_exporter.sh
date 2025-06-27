# https://github.com/google/cadvisor/releases
version="latest"

if ! id -u "cadvisor" &>/dev/null; then
	useradd -r -s /bin/false cadvisor
fi

# Create systemd service file
cat << EOF > /etc/systemd/system/cadvisor.service
[Unit]
Description=cAdvisor
After=docker.service
Requires=docker.service

[Service]
TimeoutStartSec=0
ExecStartPre=-/usr/bin/docker kill cadvisor
ExecStartPre=-/usr/bin/docker rm cadvisor
ExecStartPre=/usr/bin/docker pull "gcr.io/cadvisor/cadvisor:v$version"
ExecStart=/usr/bin/docker run \
	--volume=/:/rootfs:ro \
	--volume=/var/run:/var/run:rw \
	--volume=/cgroup:/cgroup:ro \
	--volume=/sys:/sys:ro \
	--volume=/var/lib/docker/:/var/lib/docker:ro \
	--publish=8095:8080 \
	--privileged=true
	--detach=true \
	--name=cadvisor \
	gcr.io/cadvisor/cadvisor:latest

ExecStop=/usr/bin/docker stop cadvisor

# Low CPU priority
CPUSchedulingPriority=10

[Install]
WantedBy=multi-user.target
EOF

# Start and enable cadvisor service
systemctl daemon-reload
systemctl enable cadvisor
systemctl start cadvisor
