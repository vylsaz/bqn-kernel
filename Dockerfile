FROM rust:latest
RUN useradd -m test
RUN apt update && apt install -y clang jupyter-notebook

WORKDIR /home/test
RUN git clone --recurse-submodules --depth 1 -b develop https://github.com/dzaima/CBQN.git
WORKDIR /home/test/CBQN
RUN make shared-o3
RUN cp libcbqn.so /lib/libcbqn.so

WORKDIR /home/test
RUN mkdir -p .local/share/jupyter/kernels/bqn
RUN chown -R test ./.local
COPY ./Cargo.toml ./Cargo.toml
COPY ./src/       ./src/
COPY ./bqn/       ./bqn/
RUN cargo run
RUN cp -r ./bqn/  ./.local/share/jupyter/kernels/

USER test
CMD ["jupyter", "notebook", "--debug", "--ip", "0.0.0.0"]
