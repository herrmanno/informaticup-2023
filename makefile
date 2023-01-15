all: compile

docker:
	docker build -t informaticup2023:$$(git rev-parse HEAD|cut -c-8) -t informaticup2023:latest .

run-docker: guard-task
	docker run -i --rm --network none --cpus 2.000 --memory 2G --memory-swap 2G informaticup2023:latest < $(task)

compile:
	cargo build --release

guard-%:
	@ if [ "${${*}}" = "" ]; then \
		echo "Environment variable $* not set"; \
		exit 1; \
	fi
	
zip:
	zip -r off_by_one.zip \
		common \
		inputs \
		model \
		printer \
		qa \
		simulator \
		solver \
		.dockerignore \
		.gitignore \
		Cargo.lock \
		Cargo.toml \
		Dockerfile \
		makefile \
		README.md

tar:
	tar -czf off_by_one.tar.gz \
		common \
		inputs \
		model \
		printer \
		qa \
		simulator \
		solver \
		.dockerignore \
		.gitignore \
		Cargo.lock \
		Cargo.toml \
		Dockerfile \
		makefile \
		README.md
