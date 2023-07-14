BUILDVERSION:=latest
DOCKERIMAGE:=stackmap-wmts:$(BUILDVERSION)

#get-docs:
#	go get -u github.com/swaggo/swag/cmd/swag

#docs: get-docs
#	swag init --dir cmd/api --parseDependency --output docs
.PHONY: kind-load
.PHONY: build-docker

build:
	cargo build
	
build-release:
	cargo build --release

run:
	cargo run

docker-build:
	docker build . -t $(DOCKERIMAGE)

docker-run: build-docker
	docker run --rm -it -p 9099:9099 $(DOCKERIMAGE)

port-forward:
	kubectl port-forward svc/stackmap-wmts 9099:9099

kind-load: build-docker
	kind load docker-image $(DOCKERIMAGE)

kind-deploy: kind-load
	kubectl apply -f deployment.yaml 

kind-delete:
	kubectl delete deployments/wmts

k3s-load:
	docker save $(DOCKERIMAGE) | sudo k3s ctr images import -

k8s-deploy:
	kubectl apply -f deployment.yaml

#swagger-build:
#	swagger generate spec -i ./swagger/swagger_base.yaml -o ./swagger.yaml

swagger-serve:
	cd swagger && swagger serve --flatten --port=9009 -F=swagger swagger.yaml
