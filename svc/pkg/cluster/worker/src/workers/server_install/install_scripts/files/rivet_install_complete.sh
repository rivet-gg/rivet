# Create rivet user
if ! id -u "rivet" &>/dev/null; then
	useradd -r -s /bin/false rivet
fi

# Create systemd service file
cat << 'EOF' > /etc/systemd/system/rivet_initialize.service
[Unit]
Description=Rivet Initialize
Requires=network-online.target
After=network-online.target

[Service]
User=rivet
Group=rivet
Type=oneshot
ExecStart=/usr/bin/rivet_initialize.sh
ExecStartPost=/bin/touch /var/tmp/rivet_initialize.completed
ConditionPathExists=!/var/tmp/rivet_initialize.completed

[Install]
WantedBy=multi-user.target
EOF

# Enable initialze script to run on reboot
systemctl daemon-reload
systemctl enable rivet_initialize

if [ __INITIALIZE_IMMEDIATELY__ ] ; then
    systemctl start rivet_initialize
else
	PUBLIC_IP=$(ip -4 route get 1.0.0.0 | awk '{print $7; exit}')

    # Tell Rivet this server is done installing
	curl \
		-X POST \
		-H "Authorization: Bearer __SERVER_TOKEN__" \
		"https://__DOMAIN_MAIN_API__/provision/servers/$PUBLIC_IP/install-complete"
fi
