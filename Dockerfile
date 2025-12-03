FROM rust:1.91@sha256:4a29b0db5c961cd530f39276ece3eb6e66925b59599324c8c19723b72a423615

ARG NB_USER=jovyan
ARG NB_UID=1000
ENV USER=${NB_USER}
ENV NB_UID=${NB_UID}
ENV HOME=/home/${NB_USER}

RUN adduser --disabled-password \
    --gecos "Default user" \
    --uid ${NB_UID} \
    ${NB_USER}

RUN apt update && apt install -y clang python3 python3-pip python3-venv

ENV BQN_VERSION=0.10.0

WORKDIR /opt
RUN git clone --recurse-submodules --depth 1 -b "v${BQN_VERSION}" https://github.com/dzaima/CBQN.git
WORKDIR /opt/CBQN
RUN make shared-o3
RUN cp libcbqn.so /lib/libcbqn.so

WORKDIR ${HOME}
COPY . ${HOME}
USER root
RUN chown -R ${NB_UID} ${HOME}
USER ${NB_USER}

RUN mkdir -p .local/share/jupyter/kernels/bqn
RUN cargo run
RUN cp -r ./bqn/ ./.local/share/jupyter/kernels/

RUN python3 -m venv ${HOME}/venv
# Enable venv
ENV PATH="${HOME}/venv/bin:$PATH"
RUN python3 -m pip install --no-cache-dir notebook jupyterlab jupyterhub jupyterlab_bqn
