use std::collections::HashMap;

use crate::{
    phones::build_phone_list,
    escaped_strings::{EscapedStr, EscapedString},
};

use super::{tokens::IrToken, tokenizer::tokenize_line, IrError};

/// Data that is created in the tokenization process
/// and lasts longer than the tokenization of a single line
/// 
/// Includes:
/// - definitions
/// - variables
/// 
/// # Warning
/// If variable io is used and `free_sources` is never called on this struct,
/// a memory leak will occur
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TokenizationData<'s> {
    definitions: HashMap<&'s str, Definition<'s>>,
    variables: HashMap<&'s str, Vec<IrToken<'s>>>,
    /// A list of pointers to all strs leaked
    sources: Vec<*const str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Definition<'s> {
    Lazy(&'s str),
    Eager(Vec<IrToken<'s>>),
}

impl<'s> TokenizationData<'s> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Fetches the tokens associated with a definition's name and appends them to a given lsit
    /// 
    /// Returns an error if there is no definition of the given name
    pub fn get_definition(&self, name: &'s str, tokens: &mut Vec<IrToken<'s>>, lazy_expansions: &mut Vec<&'s str>) -> Result<(), IrError<'s>> {
        match self.definitions.get(name) {
            Some(Definition::Eager(def_tokens)) => for token in def_tokens {
                tokens.push(*token);
            },
            Some(Definition::Lazy(definition)) => {
                if lazy_expansions.contains(&name) {
                    return Err(IrError::RecursiveLazyDefiniton(name));
                }

                lazy_expansions.push(name);
                
                for token in tokenize_line(definition, self, lazy_expansions)?.0 {
                    tokens.push(token);
                }

                lazy_expansions.pop();
            }
            None => return Err(IrError::UndefinedDefinition(name)),
        }

        Ok(())
    }

    /// Sets a definition
    pub fn set_definition(&mut self, name: &'s str, content: Vec<IrToken<'s>>) {
        self.definitions.insert(name, Definition::Eager(content));
    }

    /// Sets a lazy definition
    pub fn set_lazy_definition(&mut self, name: &'s str, content: &'s str) {
        self.definitions.insert(name, Definition::Lazy(content));
    }

    /// Fetches the tokens associated with a variable's name
    /// 
    /// Returns an error if there is no variable of the given name
    pub fn get_variable<'a>(&self, name: &'a str) -> Result<&Vec<IrToken<'s>>, IrError<'a>> {
        self.variables
            .get(name)
            .ok_or(IrError::UndefinedVariable(name))
    }

    /// Tokenizes the source and leaks it to the static scope,
    /// then assigns the tokens to the given name
    /// 
    /// # Warning
    /// If `free_sources` is never called on this struct, the input will be leaked forever
    pub fn set_variable_as_ir(&mut self, name: &'s str, source: String) -> Result<(), IrError<'s>> {
        let source = self.add_source_string(source);

        let (tokens, escaped_end) = tokenize_line(source, self, &mut Vec::new())?;

        if escaped_end {
            return Err(IrError::BadEscape(None));
        }

        self.variables.insert(name, tokens);
        Ok(())
    }

    /// Escapes the source then leaks it to the static scope,
    /// then assigns the it as a list of phones to the name
    /// 
    /// # Warning
    /// If `free_sources` is never called on this struct, the input will be leaked forever
    pub fn set_variable(&mut self, name: &'s str, source: &str) {
        let source = self.add_source_escaped(EscapedString::from(source));

        self.variables.insert(
            name,
            build_phone_list(source).into_iter().map(IrToken::Phone).collect()
        );
    }

    /// Leaks a source and adds it to the sources buffer
    /// 
    /// # Warning
    /// If `free_sources` is never called on this struct, the source will be leaked forever
    fn add_source_string<'a>(&mut self, mut source: String) -> &'a str {
        // leaking and moving the source to the sources buffer allows variable to be redefined
        // and prevents self reference, however, it may also cause memory leaks
        source.shrink_to_fit();
        let source = source.leak();
        self.add_source(source);
            
        source
    }

    /// Leaks a source and adds it to the sources buffer
    /// 
    /// # Warning
    /// If `free_sources` is never called on this struct, the source will be leaked forever
    fn add_source_escaped<'a>(&mut self, mut source: EscapedString) -> EscapedStr<'a> {
        // leaking and moving the source to the sources buffer allows variable to be redefined
        // and prevents self reference, however, it may also cause memory leaks
        source.shrink_to_fit();
        let source = source.leak();
        self.add_source(source.inner());
            
        source
    }

    /// Frees all variable sources and consumes the struct
    /// 
    /// # Safety
    /// There should be no references remaining to any string in the sources buffer
    pub unsafe fn free_sources(self) {
        for source in self.sources {
            let ptr = source.cast_mut();
            unsafe { drop(Box::from_raw(ptr)); }
        }
    }

    /// Adds a source to the sources buffer so it can be freed later
    fn add_source(&mut self, source: &'s str) {
        self.sources.push(std::ptr::from_ref(source));
    }

    /// Returns a reference to the sources buffer
    /// 
    /// # Warning
    /// The internal `HashMap`s may contain references to data in the sources buffer
    /// 
    /// Do not free the sources until this struct is dropped
    pub fn sources(&self) -> &[*const str] {
        &self.sources
    }

    /// Takes the sources from another `TokenizationData`
    /// 
    /// # Safety
    /// There may still be data referencing taken sources in `other`
    /// 
    /// `other` must be dropped before the sources are freed
    pub unsafe fn take_sources_from(&mut self, other: &mut Self) {
        self.sources.append(&mut other.sources);
    }

    /// Creates a new `TokenizationData` by cloning the definitions and variables,
    /// but without copying or moving the source pointers
    /// 
    /// # Warning
    /// The cloned `HashMap`s may contain references to data in the origional sources buffer
    /// 
    /// The new `TokenizationData` must be dropped before the origional sources can be freed
    pub fn with_inserts(&self) -> Self {
        Self {
            definitions: self.definitions.clone(),
            variables: self.variables.clone(),
            sources: Vec::new(),
        }
    }
}

#[cfg(test)]
#[test]
fn test_get_variable_to_multiple_phones() {
    use crate::{keywords::{BOUND_CHAR, ESCAPE_CHAR}, phones::Phone};

    let mut tokenization_data = TokenizationData::new();
    tokenization_data.set_variable("name", &format!("ab cd e {BOUND_CHAR} fg\t\t{BOUND_CHAR}h"));
    
    assert_eq!(
        tokenization_data.get_variable("name"),
        Ok(&vec![
            IrToken::Phone(Phone::Symbol("a")),
            IrToken::Phone(Phone::Symbol("b")),
            IrToken::Phone(Phone::Bound),
            IrToken::Phone(Phone::Symbol("c")),
            IrToken::Phone(Phone::Symbol("d")),
            IrToken::Phone(Phone::Bound),
            IrToken::Phone(Phone::Symbol("e")),
            IrToken::Phone(Phone::Bound),
            IrToken::Phone(Phone::Symbol(&format!("{ESCAPE_CHAR}{BOUND_CHAR}"))),
            IrToken::Phone(Phone::Bound),
            IrToken::Phone(Phone::Symbol("f")),
            IrToken::Phone(Phone::Symbol("g")),
            IrToken::Phone(Phone::Bound),
            IrToken::Phone(Phone::Symbol(&format!("{ESCAPE_CHAR}{BOUND_CHAR}"))),
            IrToken::Phone(Phone::Symbol("h")),
        ])
    );

    unsafe { tokenization_data.free_sources() };
}