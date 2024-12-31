use crate::core::adaptive::{BlockState, GrammarStateId};
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::{err_combine_opt, ParseError};
use crate::parsable::parsed::Parsed;
use crate::parser::var_map::VarMap;
use bumpalo::Bump;
use bumpalo_try::BumpaloExtend;
use std::hash::{DefaultHasher, Hasher};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct CacheKey {
    pos: Pos,
    block: usize,     // Start of blocks ptr to usize
    rule_args: usize, //TODO
    ctx: ParserContext,
    state: GrammarStateId,
}
pub type CacheVal<'arn, 'grm, E> = PResult<Parsed<'arn, 'grm>, E>;

#[derive(Copy, Clone)]
pub struct Allocs<'arn> {
    bump: &'arn Bump,
}

impl<'arn> Allocs<'arn> {
    pub fn new(bump: &'arn Bump) -> Self {
        Self { bump }
    }

    pub fn new_leaking() -> Self {
        Self {
            bump: Box::leak(Box::new(Bump::new())),
        }
    }

    pub fn alloc<T: Copy>(&self, t: T) -> &'arn mut T {
        self.bump.alloc(t)
    }

    pub fn alloc_extend<T: Copy, I: IntoIterator<Item = T, IntoIter: ExactSizeIterator>>(
        &self,
        iter: I,
    ) -> &'arn mut [T] {
        self.bump.alloc_slice_fill_iter(iter)
    }

    pub fn alloc_extend_len<T: Copy, I: IntoIterator<Item = T>>(
        &self,
        len: usize,
        iter: I,
    ) -> &'arn mut [T] {
        let mut iter = iter.into_iter();
        let slice = self.bump.alloc_slice_fill_with(len, |_| {
            iter.next().expect("Iterator supplied too few elements")
        });
        assert!(iter.next().is_none());
        slice
    }

    pub fn try_alloc_extend_option<
        T: Copy,
        I: IntoIterator<Item = Option<T>, IntoIter: ExactSizeIterator>,
    >(
        &self,
        iter: I,
    ) -> Option<&'arn mut [T]> {
        self.bump.alloc_slice_fill_iter_option(iter)
    }

    pub fn try_alloc_extend_result<
        T: Copy,
        E,
        I: IntoIterator<Item = Result<T, E>, IntoIter: ExactSizeIterator>,
    >(
        &self,
        iter: I,
    ) -> Result<&'arn mut [T], E> {
        self.bump.alloc_slice_fill_iter_result(iter)
    }
}

pub struct ParserCacheEntry<PR> {
    pub read: bool,
    pub value: PR,
}

impl<'arn, 'grm: 'arn, Env, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, Env, E> {
    pub fn parse_cache_recurse(
        &mut self,
        mut sub: impl FnMut(&mut ParserState<'arn, 'grm, Env, E>, Pos) -> PResult<Parsed<'arn, 'grm>, E>,
        blocks: &'arn [BlockState<'arn, 'grm>],
        rule_args: VarMap<'arn, 'grm>,
        grammar_state: GrammarStateId,
        pos_start: Pos,
        context: ParserContext,
    ) -> PResult<Parsed<'arn, 'grm>, E> {
        //Check if this result is cached
        let mut args_hash = DefaultHasher::new();
        for (name, value) in rule_args.iter_cloned() {
            args_hash.write(name.as_bytes());
            args_hash.write_usize(value.as_ptr().as_ptr() as usize);
        }

        let key = CacheKey {
            pos: pos_start,
            block: blocks.as_ptr() as usize,
            rule_args: args_hash.finish() as usize,
            ctx: context,
            state: grammar_state,
        };
        if let Some(cached) = self.cache_get(&key) {
            return cached.clone();
        }

        //Before executing, put a value for the current position in the cache.
        //This value is used if the rule is left-recursive
        let mut res_recursive = PResult::new_err(E::new(pos_start.span_to(pos_start)), pos_start);
        res_recursive.add_label_explicit(Debug(pos_start.span_to(pos_start), "LEFTREC"));

        let cache_state = self.cache_state_get();
        self.cache_insert(key.clone(), res_recursive);

        //Now execute the grammar rule, taking into account left recursion
        //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
        //A quick summary
        //- First put an error value for the current (rule, position) in the cache (already done)
        //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
        //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
        //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
        let res = sub(self, pos_start);
        match res {
            POk(mut o, mut spos, mut epos, mut be) => {
                //Did our rule left-recurse? (Safety: We just inserted it)
                if !self.cache_is_read(key.clone()).unwrap() {
                    //No leftrec, cache and return
                    let res = POk(o, spos, epos, be);
                    self.cache_insert(key, res.clone());
                    res
                } else {
                    //There was leftrec, we need to grow the seed
                    loop {
                        //Insert the current seed into the cache
                        self.cache_state_revert(cache_state);
                        self.cache_insert(key.clone(), POk(o, spos, epos, be.clone()));

                        //Grow the seed
                        let new_res = sub(self, pos_start);
                        match new_res {
                            POk(new_o, new_spos, new_epos, new_be)
                                if new_epos.cmp(&epos).is_gt() =>
                            {
                                o = new_o;
                                spos = new_spos;
                                epos = new_epos;
                                be = new_be;
                            }
                            POk(_, _, _, new_be) => {
                                be = err_combine_opt(be, new_be);
                                break;
                            }
                            PErr(new_e, new_s) => {
                                be = err_combine_opt(be, Some((new_e, new_s)));
                                break;
                            }
                        }
                    }

                    //The seed is at its maximum size
                    //It should still be in the cache,
                    POk(o, spos, epos, be)
                }
            }
            res @ PErr(_, _) => {
                self.cache_insert(key, res.clone());
                res
            }
        }
    }
}
