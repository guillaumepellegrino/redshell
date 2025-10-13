
# Red Shell
A Rustacean shell language for Unix platforms.

**Notice:** This crate is currently at a very early stage: do not use it.

# Overview
The purpose of this rust crate is to provide a more natural language to work with external commands. The syntax is strongly inspired from shell script with a focus on safety.

From my own opinion: bash, dash and others are great tools when you want to pipe stuff and orchestrate commands but it lacks two things:
- security, robustness: it can blow-up in your face easilly if you forget the adequate quotes, or use global variables. Why `eval` is even allowed ? Good source of 'P1' vulnerabilities.
- syntax: what the hell is the difference between '[ ]' and '[[ ]]' ??! `switch` ? Did you mean `switch_root` ?? And of course, not all shells interpret the syntax in the same way.

Now, can we pipe and orchestrate commands in Rust ? Yes, but the standard API is pretty verbose. This crate provide macro(s) and APIs to make it easy.

# Features
- Easily create or run an `std::process::Command`.
- Execute synchronously an std::process::Command and collect its status or output.
- Execute in backgroung an std::process::Command.
- Easy pipe multiple commands.
- Collect the resulting string or raw value
- Prevent shell injection issues
- A variable can not be expanded to multiple arguments or to another command. Shell injection should not be possible.

# Syntax
Special characters are `'`, ` `, `\\t`, `{}`, `>`, `<`, `>>`.

Create a simple command:
```
cmd!("echo 'Hello, World !'");
```

Execute a simple command:
```
exec!("echo 'Hello, World !'");
```

Create a command using a variable:
```
let greeting = "Bonjour, le monde";
cmd!("echo '{greeting} !'");
```

Redirect standard output to a file:
```
cmd!("echo 'Hello, World !' > /tmp/hello");
```

Redirect standard output to a file, opened in 'append' mode:
```
cmd!("echo 'Hello, World !' >> /tmp/hello");
```

Redirect standard error to `/dev/null`:
```
cmd!("echo 'Hello, World !' 2> /dev/null");
```

Read standard input from a File:
```
cmd!("cat < /proc/cpuinfo");
```

Pipe two commands:
```
let (readpipe, writepipe) = redshell::pipe();
cmd!("echo 'Hello, World !' > {readpipe}");
cmd!("tr [a-zA-Z] [A-Za-z] < {writepipe}");
```

Write standard output to memory:
```
let memfd = redshell::memfd();
cmd!("echo 'Hello, World !' > {memfd}");
let ret = memfd.read_string();
```

Open a file from memory:
```
let cert = redshell::MemoryFile::from("<<x509 certificate content>>");
let hostname = "<<https://myhostname>>";
cmd!("curl --cert {cert} {hostname}");
```

About variables:
- Variables are formated with the `format!()` macro: https://doc.rust-lang.org/std/fmt/index.html
- A Variables is not expanded into mulitple arguments even if it contains a whitespace.
- Special character like quotes, redirections inside a variable are not processed.

About single quotes:
- Use single quotes if you want to prevent a whitespace to be interpreted as a separator between two arguments.
- When using single quotes, variables are still resolved. Use `{{` or `}}` to escape the `{` or `}` characters: https://doc.rust-lang.org/std/fmt/index.html#escaping
- When using single quotes, redirect commands (`>`, `<`, `>>`) are not interpreted.

# Code examples
```
use redshell::prelude::*;

// This simple macro is expanded to:
let cmd1 = cmd!("CC={toolchain} make -C {dir} {target}");
let cmd2 = cmd!("CC={toolchain} make -C {dir}{target}");
let cmd3 = std::process::Command::new("make")
    .env("CC", String::from(toolchain))
    .args(["-C", String::from(dir), String::from(target}]);
assert_eq!(cmd1, cmd2);
assert_eq!(cmd2, cmd3);

// It is possible to compose multiple variables into a single argument using single quotes:
let cmd1 = cmd!("make -C '{dir}/{subdir}' all");
let cmd2 = std::process::Command::new("make")
    .args(["-C", format!("{dir}/{subdir}"), "all"]);
assert_eq!(cmd1, cmd2);

// Now, execute directly the command and collect its status:
let status = exec!("cat {} {} ")

let cmd = cmd!("cat file | grep search");


let cmd = cmd!("cat file" | "grep search");
```


