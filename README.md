# nasm-lsp
A simple implementation of the Language Server Protocol for NASM

----
**Install**
* Vim/NeoVim with [LanguageClient](https://github.com/autozimu/languageclient-neovim):
  * Clone this repository with `git clone https://Clinery1/nasm-lsp.git`
  * Build with rust nightly (untested with stable) in release mode (`--release`)
  * Copy the `target/releases/asm-lsp` to the directory of your choice
  * Add the line 
    ```let g:LanguageClient_serverCommands = {'nasm': ['/path/to/asm-lsp']}```
    to your `.vimrc` or `init.vim`
  
* Recommendation:
  * Add the line `autocmd BufNewFile,BufRead *.asm  set ft=nasm` to your `.vimrc` or `init.vim` to automatically change `*.asm` to nasm filetype

----
**THIS IS A WORK IN PROGRESS, USE AT YOUR OWN RISK**
