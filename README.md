# bqn-kernel
a jupyter kernel for BQN

## build

(For now:)
```
git clone https://github.com/vylsaz/bqn-kernel.git
cd bqn-kernel
cargo run
```
Then copy the `./bqn/` folder to one of the folders listed [here](https://jupyter-client.readthedocs.io/en/latest/kernels.html#kernel-specs).

(`kernel.js`, `bqn.css` and `BQN386.ttf` only work for jupyter classic, so jupyter lab users can delete them.)

## todo
- [x] running on docker
- [x] running on actual machine
- [x] support for jupyter classic
  - [x] basic syntax highlighting
  - [x] input method
- [ ] support for jupyter lab
- [ ] Rewrite It In BQN™

maybe (big MAYBE)
- [ ] plots
- [ ] graphics
- [ ] audio
- [ ] widgets
