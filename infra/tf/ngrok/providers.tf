provider "ngrok" {
    api_key = module.secrets.values["ngrok/api_key"]
}
