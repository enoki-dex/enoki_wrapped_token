SHELL = /bin/zsh

.PHONY: all
all: clean deps install test

.PHONY: deps
.SILENT: deps
deps:
	dfx start --background
	./scripts/install_dependencies.sh

.PHONY: install
.SILENT: install
install:
	./scripts/install.sh

.PHONY: init-local
.SILENT: init-local
init-local:
	./scripts/initalize_local_balance.sh $(II_PRINCIPAL)

.PHONY: config
.SILENT: config
config:
	./scripts/update_dfx.sh

.PHONY: build
.SILENT: build
build:
	dfx canister create --all
	dfx build

.PHONY: test
.SILENT: test
test:
	./tests/test.sh

.PHONY: clean
.SILENT: clean
clean:
	dfx stop
	rm -fr .dfx
