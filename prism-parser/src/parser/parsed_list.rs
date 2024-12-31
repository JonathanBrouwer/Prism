use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};

#[derive(Copy, Clone)]
pub struct ParsedList<'arn, 'grm>(Option<&'arn ParsedListNode<'arn, 'grm>>);

impl<'arn, 'grm> ParsedList<'arn, 'grm> {
    pub fn new_empty() -> Self {
        Self(None)
    }
    pub fn cons(self, head: Parsed<'arn, 'grm>, allocs: Allocs<'arn>) -> Self {
        Self(Some(allocs.alloc(ParsedListNode {
            current: head,
            next: self.0,
        })))
    }
}

impl<'arn, 'grm: 'arn> IntoIterator for ParsedList<'arn, 'grm> {
    type Item = Parsed<'arn, 'grm>;
    type IntoIter = ParsedListIterator<'arn, 'grm>;

    fn into_iter(self) -> Self::IntoIter {
        ParsedListIterator(self.0, None)
    }
}

#[derive(Copy, Clone)]
struct ParsedListNode<'arn, 'grm> {
    current: Parsed<'arn, 'grm>,
    next: Option<&'arn ParsedListNode<'arn, 'grm>>,
}

#[derive(Clone)]
pub struct ParsedListIterator<'arn, 'grm>(Option<&'arn ParsedListNode<'arn, 'grm>>, Option<usize>);

impl<'arn, 'grm: 'arn> Iterator for ParsedListIterator<'arn, 'grm> {
    type Item = Parsed<'arn, 'grm>;

    fn next(&mut self) -> Option<Self::Item> {
        let node: &'arn ParsedListNode<'arn, 'grm> = self.0?;
        self.0 = node.next;
        self.1 = self.1.map(|v| v - 1);
        Some(node.current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.1.unwrap_or_else(|| self.clone().count());
        (count, Some(count))
    }
}

impl ExactSizeIterator for ParsedListIterator<'_, '_> {}

impl<'arn, 'grm> ParseResult<'arn, 'grm> for ParsedList<'arn, 'grm> {}
impl<'arn, 'grm, Env> Parsable2<'arn, 'grm, Env> for ParsedList<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "Cons" => {
                assert_eq!(_args.len(), 2);
                _args[1]
                    .into_value::<ParsedList<'arn, 'grm>>()
                    .cons(_args[0], _allocs)
            }
            "Nil" => ParsedList::new_empty(),
            _ => unreachable!(),
        }
    }
}
