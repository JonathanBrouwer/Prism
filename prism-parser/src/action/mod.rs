use std::iter;
use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;

pub mod action_result;
pub mod apply_action;

pub trait ActionVisitor<'arn, 'grm> {
    fn visit_input_str(&mut self, s: &'arn str, span: Span, allocs: Allocs<'arn>);
    fn visit_literal(&mut self, lit: EscapedString<'grm>, allocs: Allocs<'arn>);
    fn visit_construct<'a>(&'a mut self, name: &'grm str, arity: usize, allocs: Allocs<'arn>) -> Vec<&'a mut (dyn ActionVisitor<'arn, 'grm> + 'a)>;
    fn visit_guid(&mut self, guid: usize, allocs: Allocs<'arn>);

    fn cache(&self) -> *const ();
    fn visit_cache(&mut self, value: *const ());
}

#[derive(Copy, Clone)]
pub struct IgnoreVisitor;
impl<'arn, 'grm> ActionVisitor<'arn, 'grm> for IgnoreVisitor {
    fn visit_input_str(&mut self, s: &'arn str, span: Span, allocs: Allocs<'arn>) {
    }

    fn visit_literal(&mut self, lit: EscapedString<'grm>, allocs: Allocs<'arn>) {
    }

    fn visit_construct<'a>(&'a mut self, name: &'grm str, arity: usize, allocs: Allocs<'arn>) -> Vec<&'a mut (dyn ActionVisitor<'arn, 'grm> + 'a)> {
        iter::from_fn(|| Some(allocs.alloc(IgnoreVisitor) as &mut dyn ActionVisitor)).take(arity).collect()
    }

    fn visit_guid(&mut self, guid: usize, allocs: Allocs<'arn>) {
    }

    fn cache(&self) -> *const () {
        &() as *const ()
    }

    fn visit_cache(&mut self, value: *const ()) {
    }
}

pub struct ManyVisitor<'a, 'arn, 'grm>(Vec<&'a mut dyn ActionVisitor<'arn, 'grm>>);
impl<'visitor_map, 'arn, 'grm> ActionVisitor<'arn, 'grm> for ManyVisitor<'visitor_map, 'arn, 'grm> {
    fn visit_input_str(&mut self, s: &'arn str, span: Span, allocs: Allocs<'arn>) {
        for visitor in &mut self.0 {
            visitor.visit_input_str(s, span, allocs)
        }
    }

    fn visit_literal(&mut self, lit: EscapedString<'grm>, allocs: Allocs<'arn>) {
        for visitor in &mut self.0 {
            visitor.visit_literal(lit, allocs)
        }
    }

    fn visit_construct<'a>(&'a mut self, name: &'grm str, arity: usize, allocs: Allocs<'arn>) -> Vec<&'a mut (dyn ActionVisitor<'arn, 'grm> + 'a)> {
        let mut sub_vecs = self.0.iter_mut().map(|v| v.visit_construct(name, arity, allocs).into_iter()).collect::<Vec<_>>();
        let mut result: Vec<&'a mut dyn ActionVisitor> = vec![];
        for _ in 0..arity {
            let v = sub_vecs.iter_mut().map(|it| it.next().unwrap()).collect::<Vec<_>>();
            result.push(allocs.alloc_unchecked(ManyVisitor(v)));
        }
        result
    }

    fn visit_guid(&mut self, guid: usize, allocs: Allocs<'arn>) {
        for visitor in &mut self.0 {
            visitor.visit_guid(guid, allocs)
        }
    }

    fn cache(&self) -> *const () {
        self.0[0].cache()
    }

    fn visit_cache(&mut self, value: *const ()) {
        for visitor in &mut self.0 {
            visitor.visit_cache(value)
        }
    }
}