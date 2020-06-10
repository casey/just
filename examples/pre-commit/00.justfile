# use with https://github.com/casey/just

# Example combining just + pre-commit
# pre-commit: https://pre-commit.com/
# > A framework for managing and maintaining
# > multi-language pre-commit hooks.

# pre-commit brings about encapsulation of your
# most common repo scripting tasks. It is perfectly
# usable without actually setting up precommit hooks.
# If you chose to, this justfiles include shorthands
# for git commit and amend to keep pre-commit out of
# the way when in flow on a feature branch.

# uses: https://github.com/tekwizely/pre-commit-golang 
# uses: https://github.com/prettier/prettier (pre-commit hook)
# configures: https://www.git-town.com/ (setup receipt) 

# fix auto-fixable lint issues in staged files
fix:
	pre-commit run go-returns  # fixes all Go lint issues
	pre-commit run prettier    # fixes all Markdown (& other) lint issues

# lint most common issues in - or due - to staged files
lint:
	pre-commit run go-vet-mod || true  # runs go vet
	pre-commit run go-lint    || true  # runs golint
	pre-commit run go-critic  || true  # runs gocritic

# lint all issues in - or due - to staged files:
lint-all:
	pre-commit run golangci-lint-mod || true  # runs golangci-lint

# run tests in - or due - to staged files
test:
	pre-commit run go-test-mod || true  # runs go test

# commit skipping pre-commit hooks
commit m:
	git commit --no-verify -m "{{m}}"

# amend skipping pre-commit hooks
amend:
	git commit --amend --no-verify

# install/update code automation (prettier, pre-commit, goreturns, lintpack, gocritic, golangci-lint)
install:
	npm i -g prettier
	curl https://pre-commit.com/install-local.py | python3 -
	go get github.com/sqs/goreturns
	go get github.com/go-lintpack/lintpack/...
	go get github.com/go-critic/go-critic/...
	curl -sfL https://raw.githubusercontent.com/golangci/golangci-lint/master/install.sh| sh -s -- -b $(go env GOPATH)/bin v1.27.0

# setup/update pre-commit hooks (optional)
setup:
	pre-commit install --install-hooks # uninstall: `pre-commit uninstall`
	git config git-town.code-hosting-driver gitea  # setup git-town with gitea
	git config git-town.code-hosting-origin-hostname gitea.example.org  # setup git-town origin hostname
