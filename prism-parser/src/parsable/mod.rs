use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parsable::void::Void;
use parsed::Parsed;
use std::any::type_name;

pub mod action_result;
pub mod env_capture;
pub mod guid;
pub mod option;
pub mod parsable_dyn;
pub mod parsed;
pub mod parsed_debug;
pub mod parsed_mut;
pub mod void;

pub trait ParseResult<'arn, 'grm: 'arn>: Sized + Sync + Send + Copy + 'arn {
    fn to_parsed(&'arn self) -> Parsed<'arn, 'grm> {
        Parsed::from_value(self)
    }
}

pub trait SimpleParsable<'arn, 'grm: 'arn, Env>:
    ParseResult<'arn, 'grm> + Sized + Sync + Send + Copy + 'arn
{
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        panic!(
            "Cannot parse a {} from a {constructor} constructor",
            type_name::<Self>()
        )
    }
}

pub trait ComplexParsable<'arn, 'grm: 'arn, Env>:
    ParseResult<'arn, 'grm> + Sized + Sync + Send + Copy + 'arn
{
    type Builder: ParseResult<'arn, 'grm>;

    fn build(
        constructor: &'grm str,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> Self::Builder;

    fn arg(
        s: &mut Self::Builder,
        arg: usize,
        value: Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    );

    fn finish(
        s: &mut Self::Builder,
        span: Span,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> Self;
}

#[derive(Copy, Clone)]
pub struct ComplexStore<'arn, 'grm: 'arn> {
    constructor: &'grm str,
    args: [Parsed<'arn, 'grm>; 8],
    args_len: usize,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ComplexStore<'arn, 'grm> {}

impl<'arn, 'grm: 'arn, Env, T: SimpleParsable<'arn, 'grm, Env>> ComplexParsable<'arn, 'grm, Env>
    for T
{
    type Builder = ComplexStore<'arn, 'grm>;

    fn build(
        constructor: &'grm str,
        allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self::Builder {
        ComplexStore {
            constructor,
            args: [(&Void).to_parsed(); 8],
            args_len: 0,
        }
    }

    fn arg(
        s: &mut Self::Builder,
        arg: usize,
        value: Parsed<'arn, 'grm>,
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) {
        s.args[arg] = value;
        s.args_len = s.args_len.max(arg + 1);
    }

    fn finish(
        s: &mut Self::Builder,
        span: Span,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) -> Self {
        T::from_construct(span, s.constructor, &s.args[..s.args_len], allocs, src, env)
    }
}

#[cfg(test)]
mod tests {
    use crate::parsable::ParseResult;

    #[derive(Debug, Copy, Clone)]
    struct A;
    #[derive(Debug, Copy, Clone)]
    struct B;
    impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for A {}
    impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for B {}

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
