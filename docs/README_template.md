# CSCSCA - Charles' Super Cool Sound Change Applier

A sound change applier based on linguistic sound change notation.

## Cool and Useful Features
- Digraphs (should be merged from single phones at the very start of the file)
- Application direction
- Expansive conditions and anti-conditions
- Definitions that can be inserted anywhere in a rule
- Automatic and manual matching for lists of phones
- Gaps of arbitrary phones in conditions (useful for harmony)
- Can get information to use in conditions at runtime (variables)
- Reasonably minimalist and simple, but also highly expressive and versatile
- Usable as a crate that can be adapted to fit many mediums beyond CLI

## Drawbacks
- No built-in support for syllables or suprasegmentals
- Rules must be written on a single line
- Does not include chain shift syntax (must be written as multiple rules or with scopes)

[Writing Rules]


## Command Line Interface
All CLI commands are preceded by the path to CSCSCA's executable binary.
Below this is represented with `cscsca`

### cscsca help
Prints this file

### cscsca new *`path`*
Creates a new file at *`path`*.sca

If **`-t`** or **`--template`** proceeds *`path`*, the new file has general defintions in it

### cscsca sca *`file`* *`text`*
Applies the rules in *`file`* to *`text`* and prints the result

After *`file`*, you may add a series of **`--chain`** *`file`* or **`-c`** *`file`* commands to chain the output of one file into the input of the next

Add one of the following map flags:
- `--map_outputs` or `-o` to write each output with its input and all intermediate steps between files
- `--map_prints` or `-p` to write each print output
- `--map_all` or `-m` to write each output with its input and all intermediate steps, including prints

Add **`--reduce`** or **`-x`** to remove consecutive dupicates in the output chain

Add **`--separator`** *`sep`* or **`-s`** *`sep`* after any of the map flags or reduce flag to change the default mapping separator from **`->`** to *`sep`*

Add **`--quiet`** or **`-q`** to not print logs

Add **`--write`** *`write_file`* or **`-w`** *`write_file`* before *`text`* to write the final output to *`write_file`*

Replace *`text`* with **`--read`** *`read_file`* or **`-r`** *`read_file`* to read each line of *`read_file`* as an individual input text

### cscsca chars *`text`*
`á` is not `á`. The first is `a` and the combining character `\u{301}`, the second is a single character `á`. CSCSCA counts these as different. To ensure you know which characters you are using, cscsca chars *`text`* prints every character in *`text`*, separating combining characters

## Crate Features
- `async_io`: Allows for IO to be done through asynchronous functions instead of synchronous ones. Cannot be active when compiling CSCSCA to an executable 
- `docs`: Adds the function `cscsca::docs` that returns everything under the heading `Writing Sound Change Rules With CSCSCA` in this file

## Library API
### Fallible and Infallible Application
There are both fallible and infallible variants of the crate's application functions. The fallible variants return a `Result<String, ScaError>` and the infallible variants format any errors into a `String` and do not distinguish between successful and failed application

### `IoGetter`s
Objects implementing the `IoGetter` trait allow you to control where and how input is fetched

The provided `CliGetter` uses standard IO and trims the input

### `Runtime`s
Objects implementing the `Runtime` trait allow you to control some of CSCSCA's runtime behavior
- Output: Allows you to control how printing works
- Infinite Loop Protection: Using the shifts `>` and `<` can create an infinite loop. To avoid this, CSCSCA provides a hard limit on the time/attempts applying a rule can take. This limit may be set via runtimes

The provided `CliRuntime` uses standard IO and has a limit of 10000 application attempts per line

The provided `LogRuntime` logs output internally, refreshes the logs before starting each group of applications, and uses the same limit as the `CliRuntime`

**Warning**:
If a time limit is used, it does require a call to fetch system time. In the case of Web Assembly, this causes a panic.

### `LineByLineExecuter`
A `LineByLineExecuter` may be constructed from any `Runtime`-`IoGetter` pair. You may then call the `apply` and `apply_fallible` methods to use the executor to build and then execute each line one at a time

**Note**:
Building refers to converting the raw text input into rules that can be easily applied

### `AppliableRules`
If building lines every time you apply a change is not ideal, you may use the function `build_rules` to convert the entire rule set to an appliable form. Then you can call the `apply` and `apply_fallible` methods to apply rules any number of times.
