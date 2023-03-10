# `dyncomp`

The `dyncomp` CLI provides context and project aware aliases for common used commands. It works through configuration files that can be both global (in common **configuration directories** like `XDG_CONFIG_HOME/dyncomp/config.json` or `~/.config/dyncomp/config.json` for Linux) or local, on the project root, using `dyncomp.json`.

## Example configuration file

Below you'll find an example configuration file, defined in the current directory. It defines three dyncomp commands: run, test and hello.

```
{
    "commands": {
        "run": "cargo run -- --first --second --third",
        "test": "cargo test -- --nocapture",
        "hello": "echo 'hello world'"
    }
}
```

Running `dyncomp run` will run `cargo run -- --first --second --third`, and so on.

In case this configuration was created in the configuration directory, i.e ~/.config/dyncomp/config.json, and you created another one on the current directory, it'll prefer the locally defined commands. Let's say the new local has:

```
{
    "commands": {
        "hello": "echo 'hello world from local config'"
    }
}
```

Now, while the `run` and `test` commands will still have the global behaviour, the `dyncomp hello` command will print on stdout "hello world from local config".
