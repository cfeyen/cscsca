# CSCSCA - Charles' Super Cool Sound Change Applier

A sound change applier based on linguistic sound change notation.

## Cool and Useful Features
- Digraphs (should be merged from single phones at the very start of the file)
- Application direction
- Expansive conditions and anti-conditions
- Definitions that can be inserted anywhere in a rule
- Automatic and manual matching for lists of phones
- Arbitrary length sections of repeated phones
- Can get information to use in conditions at runtime (variables)
- Reasonably minimalist and simple, but also highly expressive and versatile
- Usable as a crate that can be adapted to fit many mediums beyond CLI

## Drawbacks
- No built-in support for syllables or suprasegmentals
- Does not include chain shift syntax (must be written as multiple rules or with scopes)

## Writing Sound Change Rules with CSCSCA
### Phones
A phone is a group of non-special characters not separated by spaces

Examples: `a` `ts` `á` `litteraly_a_phone`

**Notes**:
- to convert an input `ts` (phones `t`, `s`) to the phone `ts` use the rule ```t s >> ts```

### Shifts

A shift tells CSCSCA how changes are to be applied and separates inputs from outputs
- **`>>`**: Left to right
- **`<<`**: Right to left
- **`>`**: Left to right, attempts to reapply the rule to the output of the last successful change
- **`<`**: Right to left, attempts to reapply the rule to the output of the last successful change

**Warning**: as it is technically possible to create an infinite loop with **`>`** or **`<`** or with zero-phone inputs, if applying changes to a single line is taking too long, CSCSCA will terminate itself and return an error

### Rules
A sound change

Structured *`input`* *`shift`* *`output`* where *`input`* and *`output`* are phones and *`shift`* is a shift token

Examples:
```cscsca
## `x` becomes `h`
x >> h

## a `t` `j` cluster becomes `c`
t j >> c

## `h` is lost
h >>
```

**Note**: a line starting with **`##`** is a comment

### Scopes
Scopes are a way to dynamically determine which phone, group of phones, or lack thereof exists in a rule.
There are three types of scopes
- optional **`(`**...**`)`**: a phone or group of phones that is optional
- selection **`{`**...**`,`**...**`}`**: a list of comma-separated phones or a group of phones that selects one phone or group of phones in that list
- repetition **`[`**...**`]`**: a phone or group of phones repeated 0 or more times. If a **`!`** is added in the scope, the scope represents the phone or group of phones before the **`!`** repeated 0 or more times, if it does not contain the phone or group of phones after the **`!`**


**Note**: repetition scopes are only allowed in conditions/anti-conditions (see: Conditions and Anti-Conditions)

Examples:
```cscsca
## `l` and `l` `j` become `j`
l (j) >> j

## `f` and `x` merge to `h`
{f, x} >> h

## `p` and `b` become `f` and `v` respectively
{p, b} >> {f, v}

## `u` becomes `y` when after `i` in a word (see: Conditions and Anti-Conditions)
u >> y / i [*] _

## `u` becomes `y` when after `i` in a word, unless a `w` is between the two (see: Conditions and Anti-Conditions)
u >> y / i [* ! w] _
```

### Labels
As seen in the example above, corresponding scopes in the input and output try to agree on what they choose. However, there are times when we want this behavior to be different than the default or expanded to conditions

To force scopes to agree on what they choose, we can use labels. A label has a name that starts with **`$`** and precedes a scope

**Note**: repetition scopes agree not in phones, but in phone count, causing agreeing repetition scopes to be the same length or shorter than the one that sets the agreement

Examples:
```cscsca
## `i` and `u` merge with preceding `h` or `x` into `j` `i` and `w` `u`
{h, x} $label{i, u} >> $label{j i, w u}
```

### Conditions and Anti-Conditions
To apply a rule conditionally, add a condition after it

A condition starts with a **`/`** and comes in two flavors: **pattern** and **equality**

| Condition Type | Structure | How it Checks |
|-|-|-|
| **Pattern** | *`before`* **`_`** *`after`* | checks if the rule's input is proceeded by *`before`* and followed by *`after`* |
| **Equality** | *`left`* **`=`** *`right`* | checks if the tokens in *`right`* match the phones in *`left`* (most useful with variables) |

A rule executes if any condition matches. To make a rule execute only if two sub-conditions apply, replace the **`/`** at the start of the second with **`&`**

If **`&!`** is used instead of **`&`**, the preceding sub-condition must succeed, and all sub-conditions up until the next **`/`** must fail for the overarching condition to succeed

Anti-Conditions (conditions that stop rules from applying) are the same as conditions, but start with **`//`** or **`/!`**, and should be placed after all conditions

Examples:
```cscsca
## `h` is lost word-initially
h >> / # _

## `h` is lost when not word-initial
h >> // # _ 

## stops are voiced intervocalically or after nasals
{p, t, k} >> {b, d, g} / {i, e, a, u, o} _ {i, e, a, u, o} / {m, n} _

## stops are voiced intervocalically but only in the `east` dialect
GET dialect Enter dialect:
{p, t, k} >> {b, d, g} / {i, e, a, u, o} _ {i, e, a, u, o} & %dialect = e a s t
```

**Note**: See **IO and Variables** for more on **`GET`** and **`%`**

### Definitions
Oftentimes, we want to group phones by attributes, while CSCSCA does not have support for class definitions, CSCSCA does allow you to define a *Definition*, which can later be inserted into your code

To define a *Definition* type **`DEFINE`** at the start of a line, followed by the name, then its contents.
To access the contents later, type the name prefixed with **`@`**

**Note**: *Definition*s are not limited to lists of phones, they may contain any arbitrary code

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

**`DEFINE`** evaluates the *Defintion* contents when defining it. To create a *Definition* that is evaluated every time it is used, replace **`DEFINE`** with **`DEFINE_LAZY`** (**`DEFINE_LAZY`** defined *Definitions* update with their contents)

Example:
```cscsca
DEFINE_LAZY C { @N, @P, @F, @A }

## using @C causes an error

DEFINE N {m, n}
DEFINE P {p, t, k}
DEFINE F {f, s, x}
DEFINE A {l, r}

## @C is now any consonant

{t, s} >> {c, ç} / _ i

## @C does not account for the new palatal consonants

DEFINE P {p, t, c, k}
DEFINE F {f, s, ç, x}

## @C now uses the new definitions of @P and @F accounts for all consonants again
```

### Special Characters
- **`*`**: represents any non-boundary phone. **`*`** may be preceded by a label to agree on which phone is represented
- **`#`**: a word boundary
- **`\`**: escapes the effects of the following character, may be used at the end of a line to continue the rule on the next line

### IO and Variables
To print the current phonetic form, type **`PRINT`** at the start of a line, followed by the message you would like to print with it

To get input at runtime, type **`GET`** *`variable_name`* *`message`* where *`message`* is what you want to display to prompt input. To access the input later prefix *`variable_name`* with **`%`**

**Note**: here the content of *`variable_name`* will be a list of phones, where each character is a phone, whitespace is bounds, and all special characters are properly escaped

You may replace **`GET`** with **`GET_AS_CODE`** to interpret the variable contents as code instead of phones

Examples:

```cscsca
GET dialect Enter dialect:

## h is lost in the northern and north-west dialects
## (**Note**: spaces in the words as each character is an individual phone)
h >> / %dialect = {n o r t h e r n, n o r t h - w e s t}

PRINT h-loss:
```



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

Add **`--separator`** *`sep`* or **`-s`** *`sep`* after any of the map flags or reduce flag to change the mapping separator from **`->`** to *`sep`*

Add **`--quiet`** or **`-q`** to not print logs

Add **`--write`** *`write_file`* or **`-w`** *`write_file`* before *`text`* to write the final output to *`write_file`*

Replace *`text`* with **`--read`** *`read_file`* or **`-r`** *`read_file`* to read each line of *`read_file`* as an individual input text

### cscsca chars *`text`*
`á` is not `á`. The first is `a` and the combining character `\u{301}`, the second is a single character `á`. CSCSCA counts these as different. To ensure you know which characters you are using, cscsca chars *`text`* prints every character in *`text`*, separating combining characters

## Crate Features
- `async_io`: Allows for IO to be done through asynchronous functions instead of synchronous ones. Cannot be active when compiling CSCSCA to an executable
- `debug_tokens`: Gives access to a high-level tokenized form of CSCSCA source code. (Mostly useful for custom editors)
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

### Context IO
`ContextRuntime` and `ContextIoGetter` are more versitile varients of `Runtime` and `IoGetter` that allow them to access and update context values when preforming IO operations

### `LineByLineExecuter`
A `LineByLineExecuter` may be constructed from any `Runtime`-`IoGetter` pair. You may then call the `apply` and `apply_fallible` methods to use the executor to build and then execute each line one at a time

**Note**:
Building refers to converting the raw text input into rules that can be easily applied

### `AppliableRules`
If building lines every time you apply a change is not ideal, you may use the function `build_rules` to convert the entire rule set to an appliable form. Then you can call the `apply` and `apply_fallible` methods to apply rules any number of times

`AppliableRules` has the `extend` and `extend_from` methods which let you add more rules to the end of the appliable format
