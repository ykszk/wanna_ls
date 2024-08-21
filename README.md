Wanna ls?

I don't wanna `ls` when:

- the directory is in a network storage because it can be slow.
- the directory contains too many files because it can clutter the terminal.

`wanna_ls` is a command that returns `EXIT_FAILURE` when I don't wanna `ls`.

# TODO
- config file

# Examples
Simple alias:
```bash
cdls () {
    \cd "$@" && wanna_ls && ls
}
```

See the command in action:
```bash
wanna_ls && echo yes || echo no
```

Debug output:
```bash
RUST_LOG=debug wanna_ls
```

# Compatibility
- linux: yes
- mac: ?
- windows: no