DOCKER ?= docker

.PHONY: default engine server test clean all
default: all

build:
	mkdir -p build

engine: build
	cargo build --release --target-dir build/

server: build
	cd apollod && go build -o ../build/apollod .

test:
	cargo test
	cd server && go test -cover ./...

clean:
	rm -rf build

docker:
	$(DOCKER) build -t apollo:latest .

all: engine server