# Syntax Highlighting

`justfile` syntax is close enough to `make` that you may want to tell your editor to use make syntax highlighting for just.

### Vim

For vim, you can put the following in `~/.vim/filetype.vim`:

```vimscript
if exists("did_load_filetypes")
  finish
endif

augroup filetypedetect
  au BufNewFile,BufRead justfile setf make
augroup END
```

### Vim and Emacs

Include the following in a `justfile` to enable syntax highlighting in vim and emacs:

```vimscript
# Local Variables:
# mode: makefile
# End:
# vim: set ft=make :
```

### Visual Studio Code

An extension for VS Code by https://github.com/skellock[skellock] is https://marketplace.visualstudio.com/items?itemName=skellock.just[available here]. (https://github.com/skellock/vscode-just[repository])

You can install it from the command line by running:

```sh
code --install-extension skellock.just
```

### Kakoune

Kakoune supports `justfile` syntax highlighting out of the box, thanks to TeddyDD.

### Other Editors

Feel free to send me the commands necessary to get syntax highlighting working in your editor of choice so that I may include them here.
