#!/usr/bin/env nu

use common.nu [output, hr-line, error]

def main [] {
    check-cosign

    let artifact_path = $env.ARTIFACT_PATH? | default ""
    if $artifact_path == "" {
        error "ARTIFACT_PATH is required"
    }

    if not ($artifact_path | path exists) {
        error $"artifact not found: ($artifact_path)"
    }

    let identity_token = $env.SIGSTORE_ID_TOKEN? | default ""
    let oidc_issuer = $env.SIGSTORE_OIDC_ISSUER? | default "https://token.actions.githubusercontent.com"
    let oidc_provider = $env.SIGSTORE_OIDC_PROVIDER? | default "github-actions"

    print $"(ansi green)Signing artifact:(ansi reset) ($artifact_path)"

    let sig_path = $"($artifact_path).sig"
    let cert_path = $"($artifact_path).pem"
    let bundle_path = $"($artifact_path).sigstore.json"

    # Build cosign args
    mut args = ["sign-blob" "--yes"]

    if $identity_token != "" {
        $args = ($args | append ["--identity-token" $identity_token])
    } else {
        $args = ($args | append ["--oidc-issuer" $oidc_issuer "--oidc-provider" $oidc_provider])
    }

    $args = ($args | append [
        "--output-signature" $sig_path
        "--output-certificate" $cert_path
        "--bundle" $bundle_path
        $artifact_path
    ])

    let result = do { cosign ...$args } | complete
    if $result.exit_code != 0 {
        print $"(ansi red)cosign output:(ansi reset)"
        print $result.stderr
        error "cosign signing failed"
    }

    print $"(char nl)(ansi green)Signature files:(ansi reset)"
    hr-line

    if ($sig_path | path exists) {
        print $"(ansi green)Signature:(ansi reset) ($sig_path)"
        output "signature_path" $sig_path
    }

    if ($cert_path | path exists) {
        print $"(ansi green)Certificate:(ansi reset) ($cert_path)"
        output "certificate_path" $cert_path
    }

    if ($bundle_path | path exists) {
        print $"(ansi green)Bundle:(ansi reset) ($bundle_path)"
        output "bundle_path" $bundle_path
    }

    output "artifact_path" $artifact_path
}

# Checks that cosign is available, installs if missing
def check-cosign [] {
    if (which cosign | is-empty) {
        print $"(ansi yellow)cosign not found, installing...(ansi reset)"
        let arch = match (^uname -m | str trim) {
            "arm64" | "aarch64" => "arm64"
            _ => "amd64"
        }
        let os = match (^uname -s | str trim | str downcase) {
            "darwin" => "darwin"
            _ => "linux"
        }
        let cosign_version = "2.4.1"
        let url = $"https://github.com/sigstore/cosign/releases/download/v($cosign_version)/cosign-($os)-($arch)"
        http get $url | save -f /tmp/cosign
        chmod +x /tmp/cosign
        if (which sudo | is-not-empty) {
            sudo mv /tmp/cosign /usr/local/bin/cosign
        } else {
            mv /tmp/cosign /usr/local/bin/cosign
        }
    }
}
