# bqn-kernel
a jupyter kernel for BQN

## build

Require: libcbqn on [this commit](https://github.com/dzaima/CBQN/tree/0cd1ea9bdc02652fc800d49dc672fb1119cdcbe3) or newer.

(For now:)
```
git clone https://github.com/vylsaz/bqn-kernel.git
cd bqn-kernel
```
For Windows, copy `cbqn.dll` and `cbqn.lib` to the current folder.

For Linux, make sure the dynamic linker knows where `libcbqn.so` is.

Then run:
```
cargo run
```
Then copy the `./bqn/` folder to one of the folders listed [here](https://jupyter-client.readthedocs.io/en/latest/kernels.html#kernel-specs).

(`kernel.js`, `bqn.css` and `BQN386.ttf` only work for jupyter classic, so jupyter lab users can delete them.)

## quirks 

### system values
- `•Out`, `•Show` and `•GetLine` work.
- `•platform.environment` will report `"jupyter"`.
- `•jupyter`:
  - `GetInput`: `𝕩` is prompt
  - `GetPassword`: `𝕩` is prompt
  - `HTML`: displays html (`𝕩`)
  - `Text`: displays text (`𝕩`)
  - `Clear`: clears the display

### cell magic
Start a cell with `)` to use magic. They need to be on their own lines.

#### Don't print the final result
```
)r
```
#### Use a REPL function for the cell
```
)use Func
```
`Func` needs to be able to accept a (multiline) string as input and output a value.
`Func` also needs to be able to evaluate `"•fmt"`.

## todo
- [x] running on docker
- [x] running on actual machine
- [x] support for jupyter classic
  - [x] basic syntax highlighting
  - [x] input method
- [x] [support](https://github.com/vylsaz/jupyterlab-bqn) for jupyter lab
- [ ] Rewrite It In BQN™

maybe (big MAYBE)
- [ ] plots
- [ ] graphics
- [ ] audio
- [ ] widgets
