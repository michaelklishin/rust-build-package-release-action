#!/usr/bin/env nu

# Gets the latest release version from GitHub

use common.nu [error, output]

def main [] {
    let version_override = $env.VERSION? | default ""

    let version = if $version_override != "" {
        $version_override
    } else {
        let repo = $env.GITHUB_REPOSITORY? | default ""
        if $repo == "" {
            error "GITHUB_REPOSITORY not set"
        }

        print $"(ansi green)Fetching latest release from(ansi reset) ($repo)"

        # Use gh CLI if token available (better rate limiting), otherwise curl
        let gh_token = $env.GITHUB_TOKEN? | default ($env.GH_TOKEN? | default "")
        let tag = if $gh_token != "" {
            let result = do { gh release view --repo $repo --json tagName } | complete
            if $result.exit_code != 0 {
                error $"failed to fetch release: ($result.stderr)"
            }
            ($result.stdout | from json).tagName? | default ""
        } else {
            let api_url = $"https://api.github.com/repos/($repo)/releases/latest"
            let result = do { curl -fsSL $api_url } | complete
            if $result.exit_code != 0 {
                error $"failed to fetch release: ($result.stderr)"
            }
            ($result.stdout | from json).tag_name? | default ""
        }

        if $tag == "" {
            error "no releases found"
        }

        # Strip 'v' prefix if present
        if ($tag | str starts-with "v") {
            $tag | str substring 1..
        } else {
            $tag
        }
    }

    # Validate version format (should start with digit)
    let first_char = $version | str substring 0..1
    if not ($first_char =~ '^\d$') {
        error $"invalid version format: ($version)"
    }

    print $"(ansi green)Version:(ansi reset) ($version)"
    output "version" $version
}
