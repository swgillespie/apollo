.PHONY: default engine server test clean all
default: all

build:
	mkdir -p build

engine: build
	cargo build --release --target-dir build/

server: build
	cd server && go build -o ../build/apollo-server .

test:
	cargo test
	cd server && go test -cover ./...

clean:
	rm -rf build

docker:
	docker build -t apollo:latest .

all: engine server