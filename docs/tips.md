Tips
====

## Use from Vim

If you [install Shiba with npm](installation.md), `shiba` command is available to start Shiba from Vim.  Neither Vim plugin nor specific configuration is needed.  Simply run below command in your Vim.

```sh
:!shiba --detach %
```

Shiba opens the window and starts to watch and preview the current buffer.


## Use from Emacs

You can open Shiba from Emacs by adding following code to your `.emacs`:

```lisp
(defun open-with-shiba ()
  "open a current markdown file with shiba"
  (interactive)
  (start-process "shiba" "*shiba*" "shiba" "--detach" buffer-file-name))
(define-key markdown-mode-map (kbd "C-c C-c") 'open-with-shiba)
```

Simply type `C-c C-c` and you can preview your markdown file with Shiba.


## Shiba may consume CPU power

In directories which contain so many files and directories, they cost CPU power because of so many targets to be watched.  Please consider using [`ignore_path_pattern`](customization.md) to ignore such directories.  For example, if you are developing something with Node.js, `node_modules` directory may contain so many files and you might ignore it to save CPU power.

## How to Disable Linter

If you only want a preview, you can customize to disable a linter as below with [YAML configuration file](customization.md)

```yaml
linter: none
```


-----------------
[installation](installation.md) | [usage](usage.md) | [customization](customization.md) | [shortcuts](shortcuts.md) | [tips](tips.md)
