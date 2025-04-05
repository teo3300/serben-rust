update-run:
	git pull
	cp docker-compose.yml ..
	cd ..
	docker-compose down
	docker image rm git-serben-rust:latest
	docker-compose up -d
	cd serben-rust