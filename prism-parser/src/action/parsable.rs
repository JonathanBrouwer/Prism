use std::any::type_name;
use std::hash::{DefaultHasher, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;
use crate::core::adaptive::RuleId;
use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;

pub trait Parsable<'arn, 'grm>: Sized + Sync + Send {
    fn from_span(_span: Span, _text: &'arn str, _allocs: Allocs<'arn>) -> Self {
        panic!("Cannot parse a {} from a span", type_name::<Self>())
    }

    fn from_literal(_literal: EscapedString<'grm>, _allocs: Allocs<'arn>) -> Self {
        panic!("Cannot parse a {} from a literal", type_name::<Self>())
    }

    fn from_guid(_guid: usize, _allocs: Allocs<'arn>) -> Self {
        panic!("Cannot parse a {} from a guid", type_name::<Self>())
    }

    fn from_rule(_rule: RuleId, _allocs: Allocs<'arn>) -> Self {
        panic!("Cannot parse a {} from a rule id", type_name::<Self>())
    }

    fn from_construct(_span: Span, constructor: &'grm str, _args: &[Parsed<'arn>], _allocs: Allocs<'arn>) -> Self {
        panic!("Cannot parse a {} from a {constructor} constructor", type_name::<Self>())
    }

    fn to_parsed(&self) -> Parsed<'_> {
        Parsed {
            ptr: NonNull::from(self).cast(),
            checksum: checksum_parsable::<Self>(),
            phantom_data: Default::default(),
        }
    }

    fn from_parsed(parsed: Parsed) -> &Self {
        assert_eq!(parsed.checksum, checksum_parsable::<Self>());
        unsafe {
            parsed.ptr.cast::<Self>().as_ref()
        }
    }
}

fn checksum_parsable<'arn, 'grm, P: Parsable<'arn, 'grm>>() -> u64 {
    let mut hash = DefaultHasher::new();

    hash.write_usize(P::from_span as usize);
    hash.write_usize(P::from_literal as usize);
    hash.write_usize(P::from_guid as usize);
    hash.write_usize(P::from_rule as usize);
    hash.write_usize(P::from_construct as usize);
    hash.write_usize(P::to_parsed as usize);
    hash.write_usize(P::from_parsed as usize);

    hash.finish()
}

#[derive(Copy, Clone)]
pub struct Parsed<'arn> {
    ptr: NonNull<()>,
    checksum: u64,
    phantom_data: PhantomData<&'arn ()>
}

unsafe impl<'arn> Sync for Parsed<'arn> {

}

unsafe impl<'arn> Send for Parsed<'arn> {

}

#[cfg(test)]
mod tests {
    use crate::action::parsable::Parsable;

    #[derive(Debug)]
    struct A;
    #[derive(Debug)]
    struct B;
    impl<'arn, 'grm> Parsable<'arn, 'grm> for A {
    }
    impl<'arn, 'grm> Parsable<'arn, 'grm> for B {
    }

    #[test]
    fn a_a_same() {
        let a = A;
        let parsed = a.to_parsed();
        A::from_parsed(parsed);
    }

    #[test]
    #[should_panic]
    fn a_b_different() {
        let a = A;
        let parsed = a.to_parsed();
        B::from_parsed(parsed);
    }
}