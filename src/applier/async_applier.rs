use tokio::task;

use crate::{rules::sound_change_rule::SoundChangeRule, Phone};

use super::{apply_at, next_position, ApplicationError};

/// Asynchronously applies a rule to a list of phones
// ! Should be made to remain in line with `applier::apply`
pub async fn apply<'a, 's>(rule: &'a SoundChangeRule<'s>, phones: &mut Vec<Phone<'s>>) -> Result<(), ApplicationError<'a, 's>> {
    let dir = rule.kind.dir;
    let mut phone_index = dir.start_index(phones);

    while phone_index < phones.len() {
        if let Some((replace_len, input_len)) = apply_at(rule, phones, phone_index)? {
            phone_index = next_position(rule, input_len, replace_len, phone_index, phones);
        } else {
            phone_index = dir.change_by_one(phone_index);
        }

        task::yield_now().await; // yeilds control back to the tokio runtime
        // this allows a timer to stop this function if it runs for too long
        // preventing infinite loops from being dangerous in an async enviroment
    }

    Ok(())
}