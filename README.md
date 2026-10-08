<img src="docs/banner.svg" alt=">>/ CSCSCA" width=750/>

# CSCSCA - Charles' Super Cool Sound Change Applier

A sound change applier based on linguistic sound change notation.

## Cool and Useful Features
- Digraphs (should be merged from single phones at the very start of the file)
- Unicode combining characters are automatically combined into phones
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
See: [writing_rules.md](docs/writing_rules.md)

## Command Line Interface
See: [cli.md](docs/cli.md)

## Rust Crate

See: [crate.md](docs/crate.md)

