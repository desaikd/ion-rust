#![allow(non_camel_case_types)]
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::Range;

use nom::character::streaming::satisfy;

use crate::lazy::decoder::private::LazyContainerPrivate;
use crate::lazy::decoder::{LazyRawSequence, LazyRawValue};
use crate::lazy::encoding::TextEncoding_1_0;
use crate::lazy::text::buffer::TextBufferView;
use crate::lazy::text::parse_result::AddContext;
use crate::lazy::text::parse_result::ToIteratorOutput;
use crate::lazy::text::value::{LazyRawTextValue_1_0, RawTextAnnotationsIterator};
use crate::{IonResult, IonType};

// ===== Lists =====

#[derive(Copy, Clone)]
pub struct LazyRawTextList_1_0<'data> {
    pub(crate) value: LazyRawTextValue_1_0<'data>,
}

impl<'data> LazyRawTextList_1_0<'data> {
    pub fn ion_type(&self) -> IonType {
        IonType::List
    }

    pub fn iter(&self) -> RawTextListIterator_1_0<'data> {
        let open_bracket_index = self.value.encoded_value.data_offset() - self.value.input.offset();
        // Make an iterator over the input bytes that follow the initial `[`
        RawTextListIterator_1_0::new(self.value.input.slice_to_end(open_bracket_index + 1))
    }
}

impl<'data> LazyContainerPrivate<'data, TextEncoding_1_0> for LazyRawTextList_1_0<'data> {
    fn from_value(value: LazyRawTextValue_1_0<'data>) -> Self {
        LazyRawTextList_1_0 { value }
    }
}

impl<'data> LazyRawSequence<'data, TextEncoding_1_0> for LazyRawTextList_1_0<'data> {
    type Iterator = RawTextListIterator_1_0<'data>;

    fn annotations(&self) -> RawTextAnnotationsIterator<'data> {
        self.value.annotations()
    }

    fn ion_type(&self) -> IonType {
        IonType::List
    }

    fn iter(&self) -> Self::Iterator {
        LazyRawTextList_1_0::iter(self)
    }

    fn as_value(&self) -> LazyRawTextValue_1_0<'data> {
        self.value
    }
}

impl<'a, 'data> IntoIterator for &'a LazyRawTextList_1_0<'data> {
    type Item = IonResult<LazyRawTextValue_1_0<'data>>;
    type IntoIter = RawTextListIterator_1_0<'data>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> Debug for LazyRawTextList_1_0<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for value in self {
            write!(
                f,
                "{:?}, ",
                value
                    .map_err(|_| fmt::Error)?
                    .read()
                    .map_err(|_| fmt::Error)?
            )?;
        }
        write!(f, "]").unwrap();

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RawTextListIterator_1_0<'data> {
    input: TextBufferView<'data>,
    // If this iterator has returned an error, it should return `None` forever afterwards
    has_returned_error: bool,
}

impl<'data> RawTextListIterator_1_0<'data> {
    pub(crate) fn new(input: TextBufferView<'data>) -> RawTextListIterator_1_0<'data> {
        RawTextListIterator_1_0 {
            input,
            has_returned_error: false,
        }
    }
}

impl<'data> RawTextListIterator_1_0<'data> {
    pub(crate) fn find_span(&self) -> IonResult<Range<usize>> {
        // The input has already skipped past the opening delimiter.
        let start = self.input.offset() - 1;
        // We need to find the input slice containing the closing delimiter. It's either...
        let input_after_last = if let Some(value_result) = self.last() {
            let value = value_result?;
            // ...the input slice that follows the last sequence value...
            value.input.slice_to_end(value.encoded_value.total_length())
        } else {
            // ...or there aren't values, so it's just the input after the opening delimiter.
            self.input
        };
        let (mut input_after_ws, _ws) =
            input_after_last
                .match_optional_comments_and_whitespace()
                .with_context("seeking the end of a list", input_after_last)?;
        // Skip an optional comma and more whitespace
        if input_after_ws.bytes().first() == Some(&b',') {
            (input_after_ws, _) = input_after_ws
                .slice_to_end(1)
                .match_optional_comments_and_whitespace()
                .with_context("skipping a list's trailing comma", input_after_ws)?;
        }
        let (input_after_end, _end_delimiter) = satisfy(|c| c == ']')(input_after_ws)
            .with_context("seeking the closing delimiter of a list", input_after_ws)?;
        let end = input_after_end.offset();
        Ok(start..end)
    }
}

impl<'data> Iterator for RawTextListIterator_1_0<'data> {
    type Item = IonResult<LazyRawTextValue_1_0<'data>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_returned_error {
            return None;
        }
        match self.input.match_list_value() {
            Ok((remaining, Some(value))) => {
                self.input = remaining;
                Some(Ok(value))
            }
            Ok((_remaining, None)) => {
                // Don't update `remaining` so subsequent calls will continue to return None
                None
            }
            Err(e) => {
                self.has_returned_error = true;
                e.with_context("reading the next list value", self.input)
                    .transpose()
            }
        }
    }
}

// ===== S-Expressions =====

#[derive(Copy, Clone)]
pub struct LazyRawTextSExp_1_0<'data> {
    pub(crate) value: LazyRawTextValue_1_0<'data>,
}

impl<'data> LazyRawTextSExp_1_0<'data> {
    pub fn ion_type(&self) -> IonType {
        IonType::SExp
    }

    pub fn iter(&self) -> RawTextSExpIterator_1_0<'data> {
        let open_paren_index = self.value.encoded_value.data_offset() - self.value.input.offset();
        // Make an iterator over the input bytes that follow the initial `(`
        RawTextSExpIterator_1_0::new(self.value.input.slice_to_end(open_paren_index + 1))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RawTextSExpIterator_1_0<'data> {
    input: TextBufferView<'data>,
    // If this iterator has returned an error, it should return `None` forever afterwards
    has_returned_error: bool,
}

impl<'data> RawTextSExpIterator_1_0<'data> {
    pub(crate) fn new(input: TextBufferView<'data>) -> RawTextSExpIterator_1_0<'data> {
        RawTextSExpIterator_1_0 {
            input,
            has_returned_error: false,
        }
    }
}

impl<'data> RawTextSExpIterator_1_0<'data> {
    pub(crate) fn find_span(&self) -> IonResult<Range<usize>> {
        // The input has already skipped past the opening delimiter.
        let start = self.input.offset() - 1;
        // We need to find the input slice containing the closing delimiter. It's either...
        let input_after_last = if let Some(value_result) = self.last() {
            let value = value_result?;
            // ...the input slice that follows the last sequence value...
            value.input.slice_to_end(value.encoded_value.total_length())
        } else {
            // ...or there aren't values, so it's just the input after the opening delimiter.
            self.input
        };
        let (input_after_ws, _ws) = input_after_last
            .match_optional_comments_and_whitespace()
            .with_context("seeking the end of a list", input_after_last)?;
        let (input_after_end, _end_delimiter) = satisfy(|c| c == ')')(input_after_ws)
            .with_context("seeking the closing delimiter of a list", input_after_ws)?;
        let end = input_after_end.offset();
        Ok(start..end)
    }
}

impl<'data> Iterator for RawTextSExpIterator_1_0<'data> {
    type Item = IonResult<LazyRawTextValue_1_0<'data>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_returned_error {
            return None;
        }
        match self.input.match_sexp_value() {
            Ok((remaining, Some(value))) => {
                self.input = remaining;
                Some(Ok(value))
            }
            Ok((_remaining, None)) => None,
            Err(e) => {
                self.has_returned_error = true;
                e.with_context("reading the next list value", self.input)
                    .transpose()
            }
        }
    }
}

impl<'data> LazyContainerPrivate<'data, TextEncoding_1_0> for LazyRawTextSExp_1_0<'data> {
    fn from_value(value: LazyRawTextValue_1_0<'data>) -> Self {
        LazyRawTextSExp_1_0 { value }
    }
}

impl<'data> LazyRawSequence<'data, TextEncoding_1_0> for LazyRawTextSExp_1_0<'data> {
    type Iterator = RawTextSExpIterator_1_0<'data>;

    fn annotations(&self) -> RawTextAnnotationsIterator<'data> {
        self.value.annotations()
    }

    fn ion_type(&self) -> IonType {
        IonType::SExp
    }

    fn iter(&self) -> Self::Iterator {
        LazyRawTextSExp_1_0::iter(self)
    }

    fn as_value(&self) -> LazyRawTextValue_1_0<'data> {
        self.value
    }
}

impl<'a, 'data> IntoIterator for &'a LazyRawTextSExp_1_0<'data> {
    type Item = IonResult<LazyRawTextValue_1_0<'data>>;
    type IntoIter = RawTextSExpIterator_1_0<'data>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> Debug for LazyRawTextSExp_1_0<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for value in self {
            write!(
                f,
                "{:?} ",
                value
                    .map_err(|_| fmt::Error)?
                    .read()
                    .map_err(|_| fmt::Error)?
            )?;
        }
        write!(f, ")").unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use crate::lazy::text::raw::reader::LazyRawTextReader;
    use crate::IonResult;

    fn expect_sequence_range(ion_data: &str, expected: Range<usize>) -> IonResult<()> {
        let reader = &mut LazyRawTextReader::new(ion_data.as_bytes());
        let value = reader.next()?.expect_value()?;
        let actual_range = value.encoded_value.data_range();
        assert_eq!(
            actual_range, expected,
            "Sequence range ({:?}) did not match expected range ({:?})",
            actual_range, expected
        );
        Ok(())
    }

    #[test]
    fn list_range() -> IonResult<()> {
        // For each pair below, we'll confirm that the top-level list is found to
        // occupy the specified input span.
        let tests = &[
            // (Ion input, expected range of the sequence)
            ("[]", 0..2),
            ("  []  ", 2..4),
            ("[1, 2]", 0..6),
            ("[1, /* comment ]]] */ 2]", 0..24),
            // Nested
            ("[1, 2, [3, 4, 5], 6]", 0..20),
            // Doubly nested
            ("[1, 2, [3, [a, b, c], 5], 6]", 0..28),
        ];
        for test in tests {
            expect_sequence_range(test.0, test.1.clone())?;
        }
        Ok(())
    }
}
