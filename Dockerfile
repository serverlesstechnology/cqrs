FROM rust:latest as builder

WORKDIR /home/build
RUN git clone https://github.com/serverlesstechnology/cqrs.git
WORKDIR /home/build/cqrs

