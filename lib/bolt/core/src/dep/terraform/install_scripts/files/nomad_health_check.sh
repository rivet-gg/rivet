cat << 'EOF' > /usr/local/bin/nomad_health_check.sh
#!/bin/bash
set -euf

state_file="/var/tmp/nomad_restart_state"
# In seconds
initial_backoff=1
max_backoff=300
current_time=$(date +%s)

# Read from state file
if [[ -f "$state_file" ]]; then
  read -r last_restart_time backoff_period < "$state_file"
else
  last_restart_time=0
  backoff_period=$initial_backoff
fi

time_since_last_restart=$((current_time - last_restart_time))

# Check connectivity
if ! nomad server members 2>&1 | grep -q 'alive'; then
  # Check backoff
  if [awk "$time_since_last_restart >= $backoff_period" ]; then
    echo "No Nomad servers reachable. Restarting Nomad client..."
    systemctl restart nomad

    # Increase the backoff period exponentially
    backoff_period=$((backoff_period * 2))
    if [[ $backoff_period -gt $max_backoff ]]; then
      backoff_period=$max_backoff
    fi

    # Write state
    echo "$current_time $backoff_period" > "$state_file"
  fi
else
  echo "Nomad servers reachable."

  # Write state
  echo "$current_time $initial_backoff" > "$state_file"
fi
EOF

chmod +x /usr/local/bin/nomad_health_check.sh

# Systemd service
cat << 'EOF' > /etc/systemd/system/nomad_health_check.service
[Unit]
Description=Nomad health check
Wants=network-online.target nomad.service
After=network-online.target nomad.service

[Service]
Type=oneshot
ExecStart=/usr/local/bin/nomad_health_check.sh
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

# Systemd timer
cat << 'EOF' > /etc/systemd/system/nomad_health_check.timer
[Unit]
Description=Nomad health check timer

[Timer]
OnBootSec=1min
OnUnitInactiveSec=1min
Unit=nomad_health_check.service

[Install]
WantedBy=timers.target
EOF

systemctl daemon-reload
systemctl enable nomad_health_check.timer
systemctl start nomad_health_check.timer
