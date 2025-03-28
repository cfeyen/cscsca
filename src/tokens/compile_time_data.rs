use std::collections::HashMap;

use super::{ir::IrToken, tokenize_line, IrError};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CompileTimeData<'s> {
    pub definitions: HashMap<&'s str, Vec<IrToken<'s>>>,
    variables: HashMap<&'s str, Vec<IrToken<'s>>>,
    sources: Vec<*const str>,
}

impl<'s> CompileTimeData<'s> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
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
    pub fn set_variable(&mut self, name: &'s str, mut source: String) -> Result<(), IrError<'s>> {
        source.shrink_to_fit();
        let source = source.leak();

        self.sources.push(source as *const str);

        let tokens = tokenize_line(source, self)?;

        self.variables.insert(name, tokens);
        Ok(())
    }

    /// Frees all variable sources and consumes the struct
    /// 
    /// ## Safety
    /// There should be no references remaining to any string in the source buffer
    pub unsafe fn free_sources(self) {
        self.sources.into_iter().for_each(|s| unsafe {
            (s as *mut str).drop_in_place();
        });
    }
}