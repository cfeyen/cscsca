# CSCSCA - Charles' Super Cool Sound Change Applier

A sound change applier based on linguistic sound change notation.

## Cool and Useful Features
- digraphs (should be merged from single phone at the very start of the file)
- application direction
- expansive conditions and anti-conditions
- definitions that can be inserted anywhere in a rule
- automatic and manual matching for lists of phones
- gaps of arbitrary phones in conditions (useful for harmony)
- can get information to use in conditions at runtime (variables)
- reasonably minimalist and simple but also highly expressive and versitile

## Drawbacks
- no built-in support for syllables or supersegmentals

## Writing Sound Change Rules with CSCSCA
### Phones
A phone is a group of non-special characters not seperated by spaces

Examples: `a` `ts` `á` `litteraly_a_phone`

notes:
- to convert an input *ts* (phones `t`, `s`) to the phone `ts` use the rule ```t s >> ts```

### Shifts
A shift tells the SCA how changes are to applied and seperates inputs from outputs
- `>>`: Left to right
- `<<`: Right to left
- `>`: Left to right, attempts to reapply the rule to the output of the last successful change
- `<`: Right to left, attempts to reapply the rule to the output of the last successful change

warning: as it is technically possible to create an infinite loop with `>` or `<`, if applying changes to a single line is taking too long, CSCSCA will terminate itself and return an error

### Rules
A sound change

Structured *input* *shift* *output* where *input* and *output* are phones (*input* must be at least one phone) and *shift* is a shift token

Examples:
```cscsca
## `x` becomes `h`
x >> h

## a `t` `j` cluster becomes `c`
t j >> c

## `h` is lost
h >>
```

note: a line starting with `##` is a comment

### Scopes
Scopes are a way to dynamically determine which phone, group of phones, or lack there of exists in a rule.
There are two types of scopes
- optional `(`...`)`: a phone or group of phones that is optional
- selection `{`...`,`...`}`: a list of comma-seperated phones or group of phones that selects one phone or group of phones in that list

Examples:
```cscsca
## `l` and `l` `j` become `j`
l (j) >> j

## `f` and `x` merge to `h`
{f, x} >> h

## `p` and `b` become `f` and `v` respectively
{p, b} >> {f, v}
```

### Labels
As seen in the example above corresponding scopes in the input and output try to agree in what they choose however there are times when we want this behavior to be different than the default or expanded to conditions

To force scopes to agree in what they choose, we can use labels. A label has a name that starts with `$` and proceeds a scope

Examples:
```cscsca
## `i` and `u` merge with proceeding `h` or `x` into `j` `i` and `w` `u`
{h, x} $label{i, u} >> $label{j i, w u}
```

### Conditions and Anti-Conditions
To apply rules conditionally add a condition after it

A condition starts with a `/` and comes in two flavors: **pattern** and **equality**

| Condition Type | Structure | How it Checks |
|-|-|-|
| **Pattern** | *before* `_` *after* | checks if the rule's input is proceeded by *before* and followed by *after* |
| **Match** | *left* `=` *right* | checks if the tokens in *right* match the phones in *left* (most useful with variables) |

A rule executes if any condition matches, to make a rule execute only if two sub-conditions apply replace the `/` at the start of the second with `&`

If `&!` is used instead of `&`, the proceeding sub-condition must succeed and all sub-conditions up until the next `/` must fail for the overarching condition to succeed

Anti-Conditions (conditions that stop rules from applying) are the same as conditions, but start with `//` or `/!`, and should be placed after all conditions

Examples:
```cscsca
## `h` is lost word-initially
h >> / # _

## `h` is lost when not word-initial
h >> // # _ 

## stops are voiced intervocalically or after nasals
{p, t, k} >> {b, d, g} / {i, e, a, u, o} _ {i, e, a, u, o} / {m, n} _

## stops are voiced intervocalically but using and this time
{p, t, k} >> {b, d, g} / {i, e, a, u, o} _ & _ {i, e, a, u, o}
```

### Definitions
Oftentimes we want to group phones by attributes, while CSCSCA does not have support for class definitions, CSCSCA does allow you to define a *Definition*, which can later be inserted into your code

To define a *Definition* type `DEFINE` at the start of a line, followed by the name, then its contents.
To access the contents later type the name prefixed with `@`

*Definition*s are not limitted to lists of phones, they may contain any arbitrary code

Examples:
```cscsca
DEFINE N {m, n}
DEFINE Pv- {p, t, k}
DEFINE Pv+ {b, d, g}
DEFINE V {i, e, a, u, o}

DEFINE intervocalic @V _ @V

## stops are voiced intervocalically or after nasals
## (same as the example above)
@Pv- >> @Pv+ / @intervocalic / @N _
```

### Special Characters
- `*`: represents any non-boundary phone, may be proceeded by a label to agree in what phone is represented
- `..`: a gap of zero or more non-boundary phones (notes: must have a space on both sides, only allowed in conditions), may be proceeded by a label to limit gap length to less than or equal to the length of the first gap with the same label
- `#`: a word boundary
- `\`: escapes the effects of the following character

### Reserved Characters
Characters that do nothing, but need to be escaped
- `.`
- `[`
- `]`

### IO and Variables
To print the current phonetic form, type `PRINT` at the start of a line followed by the message you would like to print with it

To get input at runtime, type `GET` *variable_name* *message* where *message* is what you want to display to prompt input. To access the input later prefix *variable_name* with `%`

note: here the content of *variable_name* will be a list phones, where each character is a phone, whitespace is bounds, and all special characters are properly escaped

You may replace `GET` with `GET_AS_CODE` to interpret the variable contents as code instead of phones

Examples:

```cscsca
GET dialect Enter dialect:

## h is lost in the northern and north-west dialects
## (note spaces in the words as each character is an individual phone)
h >> / %dialect = {n o r t h e r n, n o r t h - w e s t}

PRINT h-loss:
```


## Command Line Interface
All CLI commands are proceeded by the path to CSCSCA's executable binary.
Bellow this is represended with `cscsca`

### cscsca help
Prints the this file

### cscsca new *path*
Creates a new file at *path*.sca

If -b or --base proceeds *path*, the new file has general defintions in it

### cscsca sca *file* *text*
Applies the rules in *file* to *text* and prints the result

After *file*, you may add a series of --chain *file* or -c *file* commands to chain the output of one file into the input of the next

Add --map or -m to write each output with its input and all intermediate steps

Ass --separator *sep* or -s *sep* after --map or -m to change the default mapping separator from `->` to *sep*

Add --write *write_file* or -w *write_file* before *text* to write the final output to *write_file*

Replace *text* --read *read_file* or -r *read_file* to read each line of *read_file* as an individual input text


### cscsca gen vscode_grammar *path*
(locked behind `gen_vscode_grammar` feature)

Generates a VSCode syntax highlighting extension for `.sca`/`.cscsca` files in a directory at *path*

### cscsca chars *text*
`á` is not `á`. The first is `a` and the combining character `\u{301}`, the second is a single character `á`. CSCSCA counts these as different. To ensure you know which characters you are using, cscsca chars *text* prints every character in *text* seperating out combining characters

## Crate Features
- `ansi`: Adds ANSI color codes to IO. Ideal for CLI enviroments.
- `docs`: Adds the function `cscsca::docs` that returns everyting under the heading `Writing Sound Change Rules With CSCSCA` in this file
- `gen_vscode_grammar`: adds the `gen vscode_grammar` CLI command and exposes the function used to do so in the crate

## Library API
### Fallible and Infallible Application
There are both fallible and infallable variants of the crate's application functions. The fallible variants return a `Result<String, ScaError>` and the infallible variants format any errors into a `String` and do not distinguish between successful and failed application

### IoGetters
Objects implementing the `IoGetter` trait allow you to control where and how input is fetched

The provided `CliGetter` uses standard IO and trims the input

### Runtimes
Objects implementing the `Runtime` trait allow you to control some of CSCSCA's runtime behavior
- Output: Allows you to control how printing works
- Infinite Loop Protection: as using the shifts `>` and `<` can create an infinite loop, CSCSCA provides a hard limit on the time/attempts applying a rule can take. This limit may be set via runtimes.

The provided `CliRuntime` uses standard IO and uses a limit 10000 application attempts per line.

**Warning**:
If a time limit is used, it does require a call to fetch system time. In the case of Web Assembly, this causes a panic.

### LineByLineExecuter
A `LineByLineExecuter` may be constructed from any `Runtime`-`IoGetter` pair. You may then call the `apply` and `apply_fallible` methods to us the executer to compile then execute each line one at a time

**note**:
Here compiling does not refer to converting to machine code, rules are simply converted to the format that is used when they are applied

### AppliableRules
If compiling lines every time you apply a change is not ideal, you may use the function `compile_rules` to convert the entire rule set to an appliable form. Then you can call the `apply` and `apply_fallible` methods to apply rules any number of times.