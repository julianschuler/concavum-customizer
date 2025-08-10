use std::fmt::Display;

use itertools::Itertools;

/// A trait for indentation-aware replacing of text.
pub trait ReplaceIndented {
    /// Replaces the first occurrence of `from` in `self` with `to`.
    ///
    /// Each element of `to` is put on a separate line, matching the indentation of `from`.
    fn replace_indented(self, from: &str, to: impl IntoIterator<Item = impl Display>) -> String;
}

impl ReplaceIndented for &str {
    fn replace_indented(self, from: &str, to: impl IntoIterator<Item = impl Display>) -> String {
        if let Some(index) = self.find(from) {
            let (before, _) = self.split_at(index);

            let indent = before
                .lines()
                .last()
                .expect("there is at least one element");
            let separator = ",\n".to_owned() + indent;

            let to = to.into_iter().join(&separator);
            self.replacen(from, &to, 1)
        } else {
            self.to_string()
        }
    }
}
