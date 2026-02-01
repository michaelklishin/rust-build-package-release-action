# Contributing

## Running Tests

Run all tests:

```bash
nu tests/run.nu
```

Run a specific test file:

```bash
nu --env-config '' --config '' -I scripts tests/nu/test_common.nu
```

## Code Style

 * Format all Nu scripts with [nufmt](https://github.com/nushell/nufmt) before committing
 * Use `$env.VARIABLE?` with `| default ""` for optional env vars


## Adding New Scripts

 * Add the script to `scripts/`
 * Add tests to `tests/nu/`
 * Add command handler to `action.yml`
 * Update `AGENTS.md` key files list
 * Update `README.md` usage section
