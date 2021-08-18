$token_id = -join ((48..57) + (97..122) | Get-Random -Count 6 | ForEach-Object { [char]$_ })
$token_secret = -join ((48..57) + (97..122) | Get-Random -Count 16 | ForEach-Object { [char]$_ })

$expiration = (Get-Date).ToUniversalTime().AddHours(1).ToString("yyyy-MM-ddTHH:mm:ssZ")

@"
apiVersion: v1
kind: Secret
metadata:
  name: bootstrap-token-${token_id}
  namespace: kube-system
type: bootstrap.kubernetes.io/token
stringData:
  auth-extra-groups: system:bootstrappers:kubeadm:default-node-token
  expiration: ${expiration}
  token-id: ${token_id}
  token-secret: ${token_secret}
  usage-bootstrap-authentication: "true"
  usage-bootstrap-signing: "true"
"@ | kubectl.exe apply -f -

if (!$env:CONFIG_DIR -or -not (Test-Path $env:CONFIG_DIR)) {
  $config_dir = "$HOME\.krustlet\config"
}
else {
  $config_dir = $env:CONFIG_DIR
}

mkdir $config_dir -ErrorAction SilentlyContinue > $null

if (!$env:FILE_NAME -or -not (Test-Path $env:FILE_NAME)) {
  $file_name = "bootstrap.conf"
}