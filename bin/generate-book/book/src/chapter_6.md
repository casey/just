### GitHub Actions

[extractions/setup-just](https://github.com/extractions/setup-just) can be used to install `just` in a GitHub Actions workflow.

Example usage:

````yaml
- uses: extractions/setup-just@v1
  with:
    just-version: 0.8 # optional semver specification, otherwise latest
````