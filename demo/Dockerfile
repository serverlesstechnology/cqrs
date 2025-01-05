FROM public.ecr.aws/lambda/provided:al2
ENV RUST_BACKTRACE=1
COPY target/x86_64-unknown-linux-musl/release/bootstrap ${LAMBDA_RUNTIME_DIR}
CMD [ "cqrs.handler" ]