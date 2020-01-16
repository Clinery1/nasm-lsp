# Nasm Language Server
A simple implementation of the Language Server Protocol for NASM

----
**Install**
* Vim/NeoVim with [LanguageClient](https://github.com/autozimu/languageclient-neovim):
  * Clone this repository with `git clone https://github.com/Clinery1/nasm-lsp.git`
  * Build with rust nightly (untested with stable) in release mode (`--release`)
  * Copy the `target/releases/nasm-lsp` to the directory of your choice
  * Add the line 
    ```let g:LanguageClient_serverCommands = {'nasm': ['/path/to/nasm-lsp']}```
    to your `.vimrc` or `init.vim`
  
* Recommendation:
  * Add the line `autocmd BufNewFile,BufRead *.asm  set ft=nasm` to your `.vimrc` or `init.vim` to automatically change `*.asm` to `nasm` filetype

License
----
The entirety of this project is licensed under the BSD 3 clause liscense.
If there are any questions email me: [clinery8237@gmail.com](mailto:clinery8237@gmail.com)

----
**THIS IS A WORK IN PROGRESS, USE AT YOUR OWN RISK**
