# bqn-kernel
a jupyter kernel for BQN

[![Binder](https://mybinder.org/badge_logo.svg)](https://mybinder.org/v2/gh/vylsaz/bqn-kernel/HEAD?urlpath=%2Fdoc%2Ftree%2Findex.ipynb)

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
(add `--features v6` if you are using jupyter notebook classic)

Finally, copy the `./bqn/` folder to one of the folders listed [here](https://jupyter-client.readthedocs.io/en/latest/kernels.html#kernel-specs).

## quirks 

### kernel
The kernel cannot be interrupted.

### system values
- `•Exit` doesn't work.
- `•Out`, `•Show` and `•GetLine` work.
- `•platform.environment` will report `"jupyter"`.
- `•jupyter`:
  - `GetInput`: `𝕩` is prompt
  - `GetPassword`: `𝕩` is prompt
  - `HTML`: displays html (`𝕩`)
  - `Text`: displays text (`𝕩`)
  - `Clear`: clears the display
  - `Image`: displays image (`𝕩` of rank 2 or 3, `0.0`-`1.0` float values)
  - `Audio`: displays sound (`𝕩` of rank 1 or 2, `¯1.0`-`1.0` float values) with sample rate of `𝕨` or `audioFreq`
  - `audioFreq`: 44100

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
#### Display the final result as an image
```
)image
```
Calls `•jupyter.Image` on the final result
#### Display the final result as an audio
```
)audio
```
Calls `•jupyter.Audio` on the final result

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
- [x] image
- [x] audio
- [ ] widgets
