MAKEFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
MAKEFILE_DIR  := $(dir $(MAKEFILE_PATH))

BIN_NAME := git-ln
INSTALL_DIR := /usr/local/bin

PATH := $(MAKEFILE_DIR):$(PATH)

SUDO :=
SUDO := sudo # disable if needed by swapping the SUDO lines.

GIT_TEST_ARGS := 
GIT_TEST_ARGS := -C ~/repos/gitnu
GIT_TEST_ARGS := -C ~/repos/math

current: test

build:
	gcc main.c -o $(BIN_NAME)

build-release:
	gcc -O3 main.c -o $(BIN_NAME)

install: build-release
	$(SUDO) rm -f $(INSTALL_DIR)/$(BIN_NAME)
	$(SUDO) mv $(BIN_NAME) $(INSTALL_DIR)/$(BIN_NAME)

dev: build
	$(BIN_NAME) -n 20 --all

test: build
	git $(GIT_TEST_ARGS) ln -n 100 --all
