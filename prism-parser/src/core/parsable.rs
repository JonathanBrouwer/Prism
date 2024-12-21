use crate::core::adaptive::RuleId;
use crate::core::cache::Allocs;
use crate::core::span::Span;
use std::any::type_name;
use std::hash::{DefaultHasher, Hasher};
use std::marker::PhantomData;
use std::ptr::NonNull;

pub trait Parsable<'arn, 'grm: 'arn>: Sized + Sync + Send + 'arn {
    fn from_rule(_rule: RuleId, _allocs: Allocs<'arn>) -> Self {
        panic!("Cannot parse a {} from a rule id", type_name::<Self>())
    }

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
    ) -> Self {
        panic!(
            "Cannot parse a {} from a {constructor} constructor",
            type_name::<Self>()
        )
    }

    fn to_parsed(&'arn self) -> Parsed<'arn, 'grm> {
        Parsed {
            ptr: NonNull::from(self).cast(),
            checksum: checksum_parsable::<Self>(),
            phantom_data: Default::default(),
        }
    }

    fn try_from_parsed(parsed: Parsed<'arn, 'grm>) -> Option<&'arn Self> {
        if parsed.checksum != checksum_parsable::<Self>() {
            return None;
        }
        Some(unsafe { parsed.ptr.cast::<Self>().as_ref() })
    }
}

fn checksum_parsable<'arn, 'grm: 'arn, P: Parsable<'arn, 'grm> + 'arn>() -> u64 {
    let mut hash = DefaultHasher::new();

    hash.write_usize(P::from_rule as usize);
    hash.write_usize(P::from_construct as usize);
    hash.write_usize(P::to_parsed as usize);
    hash.write_usize(P::try_from_parsed as usize);

    hash.finish()
}

#[derive(Copy, Clone)]
pub struct Parsed<'arn, 'grm> {
    ptr: NonNull<()>,
    checksum: u64,
    phantom_data: PhantomData<(&'arn (), &'grm ())>,
}

impl<'arn, 'grm: 'arn> Parsed<'arn, 'grm> {
    pub fn into_value<P: Parsable<'arn, 'grm>>(self) -> &'arn P {
        P::try_from_parsed(self).expect("Expected wrong king of Parsable")
    }

    pub fn try_into_value<P: Parsable<'arn, 'grm>>(self) -> Option<&'arn P> {
        P::try_from_parsed(self)
    }
}

unsafe impl Sync for Parsed<'_, '_> {}

unsafe impl Send for Parsed<'_, '_> {}

pub struct Void;

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for Void {
    fn from_rule(_rule: RuleId, _allocs: Allocs<'arn>) -> Self {
        Self
    }

    fn from_construct(
        _span: Span,
        _constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
    ) -> Self {
        Self
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Guid(pub usize);

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for Guid {}

#[cfg(test)]
mod tests {
    use crate::core::parsable::Parsable;

    #[derive(Debug)]
    struct A;
    #[derive(Debug)]
    struct B;
    impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for A {}
    impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for B {}

    #[test]
    fn a_a_same() {
        let a = A;
        let parsed = a.to_parsed();
        parsed.into_value::<A>();
    }

    #[test]
    #[should_panic]
    fn a_b_different() {
        let a = A;
        let parsed = a.to_parsed();
        parsed.into_value::<B>();
    }
}
