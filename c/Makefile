BIN_NAME := git-ln

current: dev

build:
	gcc main.c -o $(BIN_NAME)

dev: build
	./$(BIN_NAME)
	@echo '────────────────────────────────────────────'
	git log --graph --format=k -n 10

install:
	gcc -O3 main.c -o $(BIN_NAME)
	mkdir -p ~/.local/bin
	rm -rf ~/.local/bin/$(BIN_NAME)
	mv $(BIN_NAME) ~/.local/bin/$(BIN_NAME)

pdev: install
	git -C ~/repos/math ln -n 100
