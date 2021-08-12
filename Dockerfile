FROM rust:alpine as builder

Copy ./ /home/build/cqrs
WORKDIR /home/build/cqrs
