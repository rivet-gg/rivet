PUBLIC_IP=$(ip -4 route get 1.0.0.0 | awk '{print $7; exit}')

# Get server info from rivet
response=$(
	curl \
		-H "Authorization: Bearer __SERVER_TOKEN__" \
		"https://__DOMAIN_MAIN_API__/provision/servers/$PUBLIC_IP/info"
)

# Fetch data
var1=$(echo $response | jq '.field1')
var2=$(echo $response | jq '.field2')
var3=$(echo $response | jq '.nestedField.field3')

# Template initialize script
initialize_script="/usr/bin/rivet_initialize.sh"
sed -i "s/__NODE_NAME__/$var1/g" $initialize_script
sed -i "s/__SERVER_ID__/$var1/g" $initialize_script
sed -i "s/__DATACENTER_ID__/$var1/g" $initialize_script
sed -i "s/__CLUSTER_ID__/$var1/g" $initialize_script
sed -i "s/__VLAN_IP__/$var1/g" $initialize_script

# Run initialize script
"$initialize_script"
