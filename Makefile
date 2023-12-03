IMAGE=espressif/idf-rust
TAG=esp32_latest
# currently: 1.74.0.0
# RUN="docker run ${IMAGE}:${TAG}"
# CARGO="${RUN} cargo"
LOCAL_IMAGE="rustdev-significant-clock"

.PHONY: docker

docker:
	docker build . -t ${LOCAL_IMAGE}
