#syntax=docker/dockerfile:1.4
ARG IMAGE="espressif/idf-rust"
ARG TAG=esp32_latest
FROM ${IMAGE}:$TAG
RUN cargo install cargo-generate
