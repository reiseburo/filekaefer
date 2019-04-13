

all: help

build: ## Build filekäfer
	cargo build

clean: ## Clean up generated code
	rm -f example.log
	cargo clean

example.log:
	while true; do date >> example.log; sleep 10; done;

run: ## Run against example.log (run `make example.log` in another console)
	 ./target/debug/filekäfer -w ./example.log

watch: ## Watch the "test" topic in Kafka with kafkacat
	kafkacat -C -b localhost:9092 -t test


# Cute hack thanks to:
# https://marmelab.com/blog/2016/02/29/auto-documented-makefile.html
help: ## Display this help text
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: all build clean run help watch
