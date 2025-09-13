use std::fmt::{Display, Write as _};

use crate::{tokens::Shift, ir::tokens::{Break, IrToken}};

use super::{tokens::RuleToken, conditions::Cond};

/// A collection of data that define a sound change rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundChangeRule<'s> {
    pub kind: Shift,
    /// The tokens that represent an input
    pub input: Vec<RuleToken<'s>>,
    /// The tokens that represent what should replace the input
    pub output: Vec<RuleToken<'s>>,
    /// The tokens that represent the enviroment in which the input should be replaced
    pub conds: Vec<Cond<'s>>,
    /// The tokens that represent the enviroment in which the input should not be replaced
    pub anti_conds: Vec<Cond<'s>>,
}

impl Display for SoundChangeRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let input = self.input
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        let output = self.output
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" ");

        let mut conds = String::new();
        for cond in &self.conds {
            _ = write!(conds, " {} {cond}", IrToken::Break(Break::Cond));
        }

        let mut anti_conds = String::new();
        for anti_cond in &self.anti_conds {
            _ = write!(anti_conds, " {} {anti_cond}", IrToken::Break(Break::AntiCond));
        }
        
        write!(f, "{} {} {}{}{}", input, &self.kind, output, conds, anti_conds)
    }
}