use std::{iter, mem, ptr};
use crate::core::adaptive::RuleId;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::serde_leak::*;
use crate::parser::var_map::VarMap;
use serde::{Deserialize, Serialize};
use crate::action::ActionVisitor;
use crate::core::cache::Allocs;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ActionResult<'arn, 'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(
        Span,
        &'grm str,
        #[serde(with = "leak_slice_of_refs")] &'arn [&'arn ActionResult<'arn, 'grm>],
    ),
    Guid(usize),
}

pub struct ActionResultVisitor<'a, 'arn, 'grm>(pub &'a mut &'arn ActionResult<'arn, 'grm>);

impl<'arn, 'grm> ActionVisitor<'arn, 'grm> for ActionResultVisitor<'_, 'arn, 'grm> {
    fn visit_input_str(&mut self, s: &'arn str, span: Span, allocs: Allocs<'arn>) {
        *self.0 = allocs.alloc(ActionResult::Value(span));
    }

    fn visit_literal(&mut self, lit: EscapedString<'grm>, allocs: Allocs<'arn>) {
        *self.0 = allocs.alloc(ActionResult::Literal(lit));
    }

    fn visit_construct<'a>(&'a mut self, name: &'grm str, arity: usize, allocs: Allocs<'arn>) -> Vec<&'a mut (dyn ActionVisitor<'arn, 'grm> + 'a)> {
        // Allocate new ActionResult::Construct
        let new_construct: &'arn mut ActionResult<'arn, 'grm> = allocs.alloc(ActionResult::Construct(Span::invalid(), name, &[]));
        let new_construct: &'a mut ActionResult<'arn, 'grm> = modify_immutable_ref(self.0, new_construct);
        let ActionResult::Construct(_, _, current_args) = new_construct else {
            unreachable!()
        };

        // Allocate new args
        let new_args: &'arn mut [&'arn ActionResult<'arn, 'grm>] = allocs.alloc_extend(iter::repeat_n(ActionResult::VOID, arity));
        let new_args: &'a mut [&'arn ActionResult<'arn, 'grm>] = modify_immutable_ref(current_args, new_args);

        // Generate visitors for args
        new_args.iter_mut().map(|action| {
            allocs.alloc_unchecked(ActionResultVisitor(action)) as &mut dyn ActionVisitor<'arn, 'grm>
        }).collect()
    }

    fn visit_guid(&mut self, guid: usize, allocs: Allocs<'arn>) {
        *self.0 = allocs.alloc(ActionResult::Guid(guid));
    }

    fn cache(&self) -> *const () {
        *self.0 as *const ActionResult as *const ()
    }

    fn visit_cache(&mut self, value: *const ()) {
        *self.0 = unsafe { &*(value as *const ActionResult<'arn, 'grm>) };
    }
}

impl<'arn, 'grm> ActionResult<'arn, 'grm> {
    pub fn get_value(&self, src: &'grm str) -> std::borrow::Cow<'grm, str> {
        match self {
            ActionResult::Value(span) => std::borrow::Cow::Borrowed(&src[*span]),
            ActionResult::Literal(s) => s.to_cow(),
            _ => panic!("Tried to get value of non-valued action result"),
        }
    }

    pub fn to_string(&self, src: &str) -> String {
        match self {
            ActionResult::Value(span) => format!("\'{}\'", &src[*span]),
            ActionResult::Literal(lit) => format!("\'{}\'", lit),
            ActionResult::Construct(_, "Cons" | "Nil", _) => {
                format!(
                    "[{}]",
                    self.iter_list()
                        .map(|e| e.to_string(src))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            ActionResult::Construct(_, c, es) => format!(
                "{}({})",
                c,
                es.iter()
                    .map(|e| e.to_string(src))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ActionResult::Guid(r) => format!("Guid({r})"),
        }
    }

    pub fn iter_list(&self) -> ARListIterator<'arn, 'grm> {
        ARListIterator(*self, None)
    }

    pub const VOID: &'static ActionResult<'static, 'static> =
        &ActionResult::Construct(Span::invalid(), "#VOID#", &[]);
}

#[derive(Clone)]
pub struct ARListIterator<'arn, 'grm: 'arn>(ActionResult<'arn, 'grm>, Option<usize>);

impl<'arn, 'grm: 'arn> Iterator for ARListIterator<'arn, 'grm> {
    type Item = &'arn ActionResult<'arn, 'grm>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            ActionResult::Construct(_, "Cons", els) => {
                assert_eq!(els.len(), 2);
                self.0 = *els[1];
                self.1 = self.1.map(|v| v - 1);
                Some(&els[0])
            }
            ActionResult::Construct(_, "Nil", els) => {
                assert_eq!(els.len(), 0);
                None
            }
            _ => panic!("Invalid list: {:?}", &self.0),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.1.unwrap_or_else(|| self.clone().count());
        (count, Some(count))
    }
}

impl ExactSizeIterator for ARListIterator<'_, '_> {}

fn modify_immutable_ref<'a, 'b, T: ?Sized>(r: &'a mut &'b T, new_value: &'b mut T) -> &'a mut T {
    // Safety: `r` is a mutable ref to an immutable ref to `T`, not sure why safe rust rejects this cast
    let old_value: *mut *const T = unsafe { mem::transmute(r) };
    let new_value: *mut T = new_value;

    // Safety: `old_value` was a mutable reference to a &'b T, so writing a value of type &'b mut T into it is sound
    unsafe { ptr::write(old_value, new_value as *const T) };

    // Safety: Since the borrow of `r` is transferred into the return value, `r` cannot be accessed until the return value is dropped
    // Therefore even though a mutable and immutable reference are alive at the same time, they cannot be accessed at the same time, so this is sound
    unsafe { &mut *new_value }
}