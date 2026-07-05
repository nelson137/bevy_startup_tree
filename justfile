set lists
set unstable

[arg('dry_run', long='dry-run', value='true')]
publish dry_run='false':
    cargo publish --workspace --exclude='*_example' --exclude='*_test_utils' {{ if dry_run == 'true' { '--dry-run' } }}
