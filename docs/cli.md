# Command Line Interface

Command Line Interface
All CLI commands are preceded by the path to CSCSCA's executable binary.
Below this is represented with `cscsca`

## cscsca help
Prints this file

## cscsca sca *`file`* *`text`*
Applies the rules in *`file`* to *`text`* and prints the result

After *`file`*, you may add a series of **`--chain`** *`file`* or **`-c`** *`file`* commands to chain the output of one file into the input of the next

Add one of the following map flags:
- `--map_outputs` or `-o` to write each output with its input and all intermediate steps between files
- `--map_prints` or `-p` to write each print output
- `--map_all` or `-m` to write each output with its input and all intermediate steps, including prints

Add **`--reduce`** or **`-x`** to remove consecutive dupicates in the output chain

Add **`--separator`** *`sep`* or **`-s`** *`sep`* after any of the map flags or reduce flag to change the mapping separator from **`->`** to *`sep`*

Add **`--quiet`** or **`-q`** to not print logs

Add **`--write`** *`write_file`* or **`-w`** *`write_file`* before *`text`* to write the final output to *`write_file`*

Replace *`text`* with **`--read`** *`read_file`* or **`-r`** *`read_file`* to read each line of *`read_file`* as an individual input text

## cscsca chars *`text`*
`á` is not `á`. The first is `a` and the combining character `\u{301}`, the second is a single character `á`. CSCSCA counts these as different. To ensure you know which characters you are using, cscsca chars *`text`* prints every character in *`text`*, separating combining characters