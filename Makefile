SHELL = /bin/bash

.PHONY: all
all: install

.PHONY: install
.SILENT: install
install: clean
	./scripts/install.sh


.PHONY: init-local
.SILENT: init-local
init-local:
	./scripts/initalize_local_balance.sh $(II_PRINCIPAL)

.PHONY: build
.SILENT: build
build:
	dfx canister create --all
	dfx build

.PHONY: test
.SILENT: test
test:
	./tests/sample_test.sh

.PHONY: clean
.SILENT: clean
clean:
	dfx stop
	rm -fr .dfx
