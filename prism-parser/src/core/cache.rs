use crate::core::adaptive::{BlockState, GrammarState};
use crate::core::context::{ParserContext, PR};
use crate::core::cow::Cow;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::{err_combine_opt, ParseError};
use crate::grammar::GrammarFile;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use by_address::ByAddress;
use std::collections::HashMap;
use typed_arena::Arena;

//TODO bug: does not include params
type CacheKey<'grm, 'b> = (Pos, (ByAddress<&'b [BlockState<'b, 'grm>]>, ParserContext));

pub struct ParserCache<'grm, 'b, E: ParseError> {
    //Cache for parser_cache_recurse
    cache: HashMap<CacheKey<'grm, 'b>, ParserCacheEntry<PResult<PR<'b, 'grm>, E>>>,
    cache_stack: Vec<CacheKey<'grm, 'b>>,
    // For allocating things that might be in the result
    pub alloc: Allocs<'b, 'grm>,
    pub input: &'grm str,
}

pub type PCache<'b, 'grm, E> = ParserCache<'grm, 'b, E>;

#[derive(Clone)]
pub struct Allocs<'b, 'grm: 'b> {
    pub alo_grammarfile: &'b Arena<GrammarFile<'grm, RuleAction<'b, 'grm>>>,
    pub alo_grammarstate: &'b Arena<GrammarState<'b, 'grm>>,
    pub alo_ar: &'b Arena<ActionResult<'b, 'grm>>,
}

impl<'b, 'grm> Allocs<'b, 'grm> {
    pub fn uncow(&self, cow: Cow<'b, ActionResult<'b, 'grm>>) -> &'b ActionResult<'b, 'grm> {
        match cow {
            Cow::Borrowed(v) => v,
            Cow::Owned(v) => self.alo_ar.alloc(v),
        }
    }
}

pub struct ParserCacheEntry<PR> {
    read: bool,
    value: PR,
}

impl<'grm, 'b, E: ParseError> ParserCache<'grm, 'b, E> {
    pub fn new(input: &'grm str, alloc: Allocs<'b, 'grm>) -> Self {
        ParserCache {
            cache: HashMap::new(),
            cache_stack: Vec::new(),
            alloc,
            input,
        }
    }

    pub(crate) fn cache_is_read(&self, key: CacheKey<'grm, 'b>) -> Option<bool> {
        self.cache.get(&key).map(|v| v.read)
    }

    pub(crate) fn cache_get(
        &mut self,
        key: CacheKey<'grm, 'b>,
    ) -> Option<&PResult<PR<'b, 'grm>, E>> {
        if let Some(v) = self.cache.get_mut(&key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    pub(crate) fn cache_insert(
        &mut self,
        key: CacheKey<'grm, 'b>,
        value: PResult<PR<'b, 'grm>, E>,
    ) {
        self.cache
            .insert(key.clone(), ParserCacheEntry { read: false, value });
        self.cache_stack.push(key);
    }

    pub(crate) fn cache_state_get(&self) -> usize {
        self.cache_stack.len()
    }

    pub(crate) fn cache_state_revert(&mut self, state: usize) {
        self.cache_stack.drain(state..).for_each(|key| {
            self.cache.remove(&key);
        })
    }

    pub(crate) fn clear(&mut self) {
        self.cache.clear();
        self.cache_stack.clear();
    }
}

pub fn parser_cache_recurse<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>>>(
    sub: &'a impl Parser<'b, 'grm, PR<'b, 'grm>, E>,
    id: (ByAddress<&'b [BlockState<'b, 'grm>]>, ParserContext),
) -> impl Parser<'b, 'grm, PR<'b, 'grm>, E> + 'a {
    move |pos_start: Pos, state: &mut PCache<'b, 'grm, E>, context: &ParserContext| {
        //Check if this result is cached
        let key = (pos_start, id.clone());
        if let Some(cached) = state.cache_get(key.clone()) {
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
                        state.cache_insert(
                            key.clone(),
                            POk(o.clone(), spos, epos, empty, be.clone()),
                        );

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
