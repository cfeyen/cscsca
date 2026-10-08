# CSCSCA Crate

## Crate Features
- `async_io`: Allows for IO to be done through asynchronous functions instead of synchronous ones. Cannot be active when compiling CSCSCA to an executable
- `debug_tokens`: Gives access to a high-level tokenized form of CSCSCA source code. (Mostly useful for custom editors)
- `sys_time`: Adds `LineApplicationLimit::Time`, which allows CSCSCA to end a long application after a time limit instead of an attempt limit. (**Warning**: this is not WASM compatable)

## Library API
### Fallible and Infallible Application
There are both fallible and infallible variants of the crate's application functions. The fallible variants return a `Result<String, ScaError>` and the infallible variants format any errors into a `String` and do not distinguish between successful and failed application

### `IoGetter`s
Objects implementing the `IoGetter` trait allow you to control where and how input is fetched

### `Runtime`s
Objects implementing the `Runtime` trait allow you to control some of CSCSCA's runtime behavior
- Output: Allows you to control how printing works
- Infinite Loop Protection: Using the shifts `>` and `<` can create an infinite loop. To avoid this, CSCSCA provides a hard limit on the time/attempts applying a rule can take. This limit may be set via runtimes

The provided `LogRuntime` logs output refreshes the logs before starting each group of applications, and uses a default limit of 10000 application attempts

**Warning**:
If a time limit is used, it does require a call to fetch system time. In the case of Web Assembly, this causes a panic.

### Context IO
`ContextRuntime` and `ContextIoGetter` are more versitile varients of `Runtime` and `IoGetter` that allow them to access and update context values when preforming IO operations

### `LineByLineExecutor`
A `LineByLineExecutor` may be constructed from any `Runtime`-`IoGetter` pair. You may then call the `apply` and `apply_fallible` methods to use the executor to build and then execute each line one at a time

**Note**:
Building refers to converting the raw text input into rules that can be easily applied

### `AppliableRules`
If building lines every time you apply a change is not ideal, or you wish to only fetch input once. You can call the `apply` and `apply_fallible` methods to apply these rules any number of times

`AppliableRules` has the `extend` methods which let you add more rules to the end of the appliable format