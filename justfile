set shell := ["nu", "-c"]

release-dry LEVEL:
    cargo release version {{LEVEL}}
    cargo release tag
    # standard-version --skip.tag --dry-run
    git-cliff

release LEVEL:
    cargo release version {{LEVEL}} -x
    cargo release hook -x
    git add .
    cargo release commit -x
    cargo release tag -x
    # standard-version --skip.tag --dry-run
    # git-cliff -o CHANGELOG.md