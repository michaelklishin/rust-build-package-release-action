# Contributing

## Running Tests

Run all tests:

```bash
nu tests/run.nu
```

Run a specific test file:

```bash
nu --env-config '' --config '' -I $(pwd)/scripts tests/nu/test_common.nu
```

**Note:** The `-I` flag requires an absolute path to the `scripts/` directory. Use `$(pwd)/scripts` or the full path. The test runner handles this automatically by constructing absolute paths from `$env.FILE_PWD`.

## Writing Tests

 * Place test files in `tests/nu/` with a `test_*.nu` naming pattern
 * Use `use std/assert` for assertions
 * Tests can import from `scripts/` using `use common.nu [functions]` - the test runner configures the include path automatically
 * When testing functions that read environment variables, use `hide-env -i VAR_NAME` to ensure defaults are tested (setting a var to `""` is different from it being unset)
 * Each test file needs a `def main []` entry point that calls individual test functions

## Code Style

 * Format all Nu scripts with [nufmt](https://github.com/nushell/nufmt) before committing
 * Use `$env.VARIABLE?` with `| default ""` for optional env vars


## Adding New Scripts

 * Add the script to `scripts/`
 * Add tests to `tests/nu/`
 * Add command handler to `action.yml`
 * Update `AGENTS.md` key files list
 * Update `README.md` usage section
