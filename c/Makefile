BIN_NAME := git-ln
INSTALL_DIR := /usr/local/bin

SUDO :=
SUDO := sudo

current: pdev

build:
	gcc main.c -o $(BIN_NAME)

dev: build
	./$(BIN_NAME)
	@echo '────────────────────────────────────────────'
	git log --graph --format=k -n 10

install:
	gcc -O3 main.c -o $(BIN_NAME)
	$(SUDO) rm -rf $(INSTALL_DIR)/$(BIN_NAME)
	$(SUDO) mv $(BIN_NAME) $(INSTALL_DIR)/$(BIN_NAME)

pdev: install
	# git -C ~/repos/math ln -n 100
	git ln -n 10 --all
