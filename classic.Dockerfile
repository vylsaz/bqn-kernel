FROM rust:latest
RUN useradd -m test
RUN apt update && apt install -y clang jupyter-notebook

WORKDIR /home/test
RUN git clone --recurse-submodules --depth 1 https://github.com/dzaima/CBQN.git
WORKDIR /home/test/CBQN
RUN make shared-o3
RUN cp libcbqn.so /lib/libcbqn.so

WORKDIR /home/test
RUN mkdir -p .local/share/jupyter/kernels/bqn
RUN chown -R test ./.local
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src/       ./src/
COPY ./bqn/       ./bqn/
COPY ./bqn-v6/    ./bqn-v6/
RUN cargo run --features v6
RUN cp -r ./bqn/  ./.local/share/jupyter/kernels/

USER test
EXPOSE 8888
CMD ["jupyter", "notebook", "--ip", "0.0.0.0"]
