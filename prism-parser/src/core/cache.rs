use crate::core::adaptive::{BlockState, GrammarState, GrammarStateId};
use crate::core::context::ParserContext;
use crate::core::cow::Cow;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::state::PState;
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::{err_combine_opt, ParseError};
use crate::grammar::GrammarFile;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use by_address::ByAddress;
use typed_arena::Arena;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct CacheKey<'grm, 'arn> {
    pos: Pos,
    block: ByAddress<&'arn [BlockState<'arn, 'grm>]>,
    ctx: ParserContext,
    state: GrammarStateId,
    params: Vec<(&'grm str, Cow<'arn, ActionResult<'arn, 'grm>>)>,
}

pub type CacheVal<'grm, 'arn, E> = PResult<&'arn ActionResult<'arn, 'grm>, E>;

#[derive(Clone)]
pub struct Allocs<'arn, 'grm: 'arn> {
    pub alo_grammarfile: &'arn Arena<GrammarFile<'grm, RuleAction<'arn, 'grm>>>,
    pub alo_grammarstate: &'arn Arena<GrammarState<'arn, 'grm>>,
    pub alo_ar: &'arn Arena<ActionResult<'arn, 'grm>>,
}

impl<'arn, 'grm> Allocs<'arn, 'grm> {
    pub fn uncow(
        &self,
        cow: Cow<'arn, ActionResult<'arn, 'grm>>,
    ) -> &'arn ActionResult<'arn, 'grm> {
        match cow {
            Cow::Borrowed(v) => v,
            Cow::Owned(v) => self.alo_ar.alloc(v),
        }
    }
}

pub struct ParserCacheEntry<PR> {
    pub read: bool,
    pub value: PR,
}

pub fn parser_cache_recurse<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>>(
    sub: &'a impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E>,
    block: ByAddress<&'arn [BlockState<'arn, 'grm>]>,
    grammar_state: GrammarStateId,
    params: Vec<(&'grm str, Cow<'arn, ActionResult<'arn, 'grm>>)>,
) -> impl Parser<'arn, 'grm, &'arn ActionResult<'arn, 'grm>, E> + 'a {
    move |pos_start: Pos, state: &mut PState<'arn, 'grm, E>, context: &ParserContext| {
        //Check if this result is cached
        let key = CacheKey {
            pos: pos_start,
            block,
            ctx: context.clone(),
            state: grammar_state,
            params: params.clone(),
        };
        if let Some(cached) = state.cache_get(&key) {
            return cached.clone();
        }

        //Before executing, put a value for the current position in the cache.
        //This value is used if the rule is left-recursive
        let mut res_recursive = PResult::new_err(E::new(pos_start.span_to(pos_start)), pos_start);
        res_recursive.add_label_explicit(Debug(pos_start.span_to(pos_start), "LEFTREC"));

        let cache_state = state.cache_state_get();
        state.cache_insert(key.clone(), res_recursive);

        //Now execute the grammar rule, taking into account left recursion
        //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
        //A quick summary
        //- First put an error value for the current (rule, position) in the cache (already done)
        //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
        //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
        //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
        let res = sub.parse(pos_start, state, context);
        match res {
            POk(mut o, mut spos, mut epos, mut empty, mut be) => {
                //Did our rule left-recurse? (Safety: We just inserted it)
                if !state.cache_is_read(key.clone()).unwrap() {
                    //No leftrec, cache and return
                    let res = POk(o, spos, epos, empty, be);
                    state.cache_insert(key, res.clone());
                    res
                } else {
                    //There was leftrec, we need to grow the seed
                    loop {
                        //Insert the current seed into the cache
                        state.cache_state_revert(cache_state);
                        state.cache_insert(key.clone(), POk(o, spos, epos, empty, be.clone()));

                        //Grow the seed
                        let new_res = sub.parse(pos_start, state, context);
                        match new_res {
                            POk(new_o, new_spos, new_epos, new_empty, new_be)
                                if new_epos.cmp(&epos).is_gt() =>
                            {
                                o = new_o;
                                spos = new_spos;
                                epos = new_epos;
                                empty = new_empty;
                                be = new_be;
                            }
                            POk(_, _, _, _, new_be) => {
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
                    POk(o, spos, epos, empty, be)
                }
            }
            res @ PErr(_, _) => {
                state.cache_insert(key, res.clone());
                res
            }
        }
    }
}
