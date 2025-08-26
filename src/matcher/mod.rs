use crate::{phones::Phone, tokens::Direction};

#[cfg(test)]
mod tests;
pub mod choices;
pub mod match_state;
pub mod pattern;
pub mod rule_pattern;

/// A directional `Iterator` over a list of phones
/// 
/// Note: does not implement `Copy` to avoid confusion with implicit copies via the `Iterator` trait
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phones<'p, 's> {
    phone_list: &'p [Phone<'s>],
    /// the index of the next phone LTR,
    /// the index after the next phone RTL,
    /// 
    /// \[p0, p1,  p2,  p3,  p4\]
    /// 
    /// index of 1 points ***between*** p0 and p1
    index: Option<usize>,
    direction: Direction,
}

impl<'p, 's> Phones<'p, 's> {
    /// Creates a new phone iterator where `index` is the index of the first phone to be returned
    pub const fn new(phones: &'p [Phone<'s>], index: usize, direction: Direction) -> Self {
        Self {
            phone_list: phones,
            index: if index <= phones.len() {
                match direction {
                    Direction::Ltr => Some(index),
                    Direction::Rtl => Some(index + 1),
                }
            } else {
                None
            },
            direction,
        }
    }

    /// Gets the next phone
    pub fn next(&mut self) -> &'p Phone<'s> {
        if let Some(i) = self.index {
            match self.direction {
                Direction::Ltr => {
                    let phone = self.phone_list.get(i);
                    self.index = i.checked_add(1);
                    phone
                },
                Direction::Rtl => {
                    if let Some(i) = i.checked_sub(1) {
                        self.index = Some(i);
                        self.phone_list.get(i)
                    } else {
                        None
                    }
                }
            }
        } else {
            None
        }.unwrap_or_default()
    }

    /// Gets the direction
    pub const fn direction(&self) -> Direction {
        self.direction
    }

    /// Gets the phones to the left of the index
    fn left(&self) -> &'p [Phone<'s>] {
        if let Some(i) = self.index {
            self.phone_list.get(..i).unwrap_or_default()
        } else {
            &[]
        }
    }

    /// Gets the phones to the right of the index
    fn right(&self) -> &'p [Phone<'s>] {
        if let Some(i) = self.index {
            self.phone_list.get(i..).unwrap_or_default()
        } else {
            &[]
        }
    }

    /// Creats an rtl `Phone` `Iterator` from everything left of the current index
    fn rtl_from_left(&self) -> Self {
        let phones = self.left();

        Self {
            phone_list: phones,
            index: Some(phones.len()),
            direction: Direction::Rtl,
        }
    }

    /// Creats an ltr `Phone` `Iterator` from everything right of the current index
    fn ltr_from_right(&self) -> Self {
        let phones = self.right();

        Self {
            phone_list: phones,
            index: Some(0),
            direction: Direction::Ltr,
        }
    }
}