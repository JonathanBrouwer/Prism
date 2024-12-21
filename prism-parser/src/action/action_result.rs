use crate::core::cache::Allocs;
use crate::core::parsable::{Parsable, Parsed};
use crate::core::span::Span;
use crate::parser::var_map::VarMap;

#[derive(Copy, Clone)]
pub enum ActionResult<'arn, 'grm> {
    Construct(Span, &'grm str, &'arn [Parsed<'arn, 'grm>]),
    WithEnv(VarMap<'arn, 'grm>, Parsed<'arn, 'grm>),
}

impl<'arn, 'grm> Parsable<'arn, 'grm> for ActionResult<'arn, 'grm> {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
    ) -> Self {
        Self::Construct(span, constructor, allocs.alloc_extend(args.iter().copied()))
    }
}

impl<'arn, 'grm> ActionResult<'arn, 'grm> {
    pub fn iter_list(&self) -> ARListIterator<'arn, 'grm> {
        ARListIterator(*self, None)
    }
}

#[derive(Clone)]
pub struct ARListIterator<'arn, 'grm: 'arn>(ActionResult<'arn, 'grm>, Option<usize>);

impl<'arn, 'grm: 'arn> Iterator for ARListIterator<'arn, 'grm> {
    type Item = Parsed<'arn, 'grm>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            ActionResult::Construct(_, "Cons", els) => {
                assert_eq!(els.len(), 2);
                self.0 = *els[1].into_value::<ActionResult<'arn, 'grm>>();
                self.1 = self.1.map(|v| v - 1);
                Some(els[0])
            }
            ActionResult::Construct(_, "Nil", els) => {
                assert_eq!(els.len(), 0);
                None
            }
            _ => panic!("Invalid list"),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.1.unwrap_or_else(|| self.clone().count());
        (count, Some(count))
    }
}

impl ExactSizeIterator for ARListIterator<'_, '_> {}
