# Merenda - Snack-sized clipboard syncronization over SSH

A tiny server/client application to synchronize clipboards over SSH written in Rust, inspired by similar tools like [Lemonade](https://github.com/pocke/lemonade).

# Installation

First, install merenda using cargo on both your local machine (the one you'll use to SSH into other systems), and your remote machine (the one you SSH into).

```
cargo install merenda
```

## 1. Local machine

After installation, run `merenda server` on your main machine (the one you'll use to SSH into other systems). It will listen for connections requesting to read/write your clipboard, and you'll generally want to have it running in the background. On its own, it also works as a roundabout way to read/write your clipboard, by using `merenda get` and `merenda set` on your local machine to connect to the server.

If you're using systemd, you can copy the [provided user service](examples/merenda.service) to `~/.config/systemd/user`, which you can activate with `systemctl --user start merenda.service` and enable at boot with `systemctl --user enable merenda.service`.

By default, the server will listen on `127.0.0.1`, port 3660. You can change either with the `-H/--address` and `-p/--port` arguments.

*CAUTION: Think carefully before changing the default listen address*. Merenda has absolutely no authentication or encryption, so listening on anything other than loopback will allow anyone able to reach that IP address to read/write to your clipboard at will. A safer alternative is to forward the port over SSH, so that it's only exposed on the other end of the SSH connection. More on that later.

## 2. Remote machine

After installation, set up your preferred editors/tools to use merenda to access the clipboard when an SSH connection is available.

For example, I use NeoVim and my config looks like this:

```
if !empty($SSH_CONNECTION)
    set clipboard+=unnamed
    let g:clipboard= {
        \   'name': 'merenda',
        \   'copy': {
        \       '*': ['merenda', 'set'],
        \   },
        \   'paste': {
        \       '*': ['merenda', 'get'],
        \   },
        \   'cache_enabled': 1,
        \ }
endif
```

## 3. Forwarding over SSH

You can forward the port merenda is listening on to your remote host by adding a `RemoteForward` option to your `~/.ssh/config` file.

For example:

```
Host myothercomputer
    User username
    RemoteForward 3660 localhost:3660
    Hostname 10.1.1.251
```

Now you can use `merenda set` and `merenda get` to read/write the clipboard from the remote host!

# License

This project and all contributions to it are licensed under the GPL General Public License v3.
