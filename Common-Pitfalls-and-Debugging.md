## A command does not behave like expected

**First step:** Try to run the command without adding it to Pueue.
Do this by calling `sh -c '$COMMAND'` instead of `pueue add '$COMMAND'`.
If this fails, it's not a problem with Pueue, but the command or the shell's features itself.

Some examples of what can go wrong:

- A single `&` somewhere in your command detaches the process, causing the Pueue task to finish immediately.
- The system shell in Ubuntu 18.04, doesn't support the `&>` parameter, which is interpreted as `&` and detaches processes. Use `2>&1` instead.


**Second step** is to look at the process output:

This can be done via `pueue log $task_id`.
You can also get a live view of the output with `pueue follow $task_id`.
Add the `-e` flag, if you want to see the error output.

### The command formatting seems to be broken

Pueue takes your input and uses it exactly as is to create a new `bash -c $command` in the background.  
If your command contains spaces or characters that need escaping, you might need to encapsulate it as a string:

```bash
pueue add -- ls -al "/tmp/this\ is\ a\ test\ directory"
```

Without quotes, the character escaping won't be transferred to the `bash -c $command`, as it's already removed by calling it from the current shell.

### A process waits for input

Sometimes some process waits for input. For instance, a package manager may wait for confirmation (`y/n`).

In this case you can send the desired input to the process via:

```bash
pueue send "y
"
```

This can be also be avoided by issuing the command with something like a `-y` flag (if the program allows something like this).

### My shell aliases don't work

Pueue doesn't support aliases in shell's `.*rc` files, since that's pretty tricky.
That's why Pueue brings it's own aliasing.
Check the [Miscellaneous section](https://github.com/Nukesor/pueue/wiki/Miscellaneous) on how to use it.

### Display not found

All programs that require some kind of display/window manager won't work, as the tasks are executed in the background.\
Don't use Pueue for commands that won't work in a non-visual environment.

