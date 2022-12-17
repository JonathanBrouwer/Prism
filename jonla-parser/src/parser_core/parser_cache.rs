use crate::parser_core::adaptive::BlockState;
use crate::parser_core::error::{err_combine_opt, ParseError};
use crate::parser_core::parser::Parser;
use crate::parser_core::presult::PResult;
use crate::parser_core::presult::PResult::{PErr, POk};
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::error_printer::ErrorLabel::Debug;
use by_address::ByAddress;
use std::collections::HashMap;
use crate::parser_sugar::parser_context::{ParserContext, PR, PCache};

type CacheKey<'grm, 'b> = (
    usize,
    (
        ByAddress<&'b [BlockState<'b, 'grm>]>,
        ParserContext<'b, 'grm>,
    ),
);

pub struct ParserCache<'grm, 'b, PR> {
    //Cache for parser_cache_recurse
    cache: HashMap<CacheKey<'grm, 'b>, ParserCacheEntry<PR>>,
    cache_stack: Vec<CacheKey<'grm, 'b>>,
}

pub struct ParserCacheEntry<PR> {
    read: bool,
    value: PR,
}

impl<'grm, 'b, PR: Clone> ParserCache<'grm, 'b, PR> {
    pub fn new() -> Self {
        ParserCache {
            cache: HashMap::new(),
            cache_stack: Vec::new(),
        }
    }

    pub(crate) fn cache_is_read(&self, key: &CacheKey<'grm, 'b>) -> Option<bool> {
        self.cache.get(key).map(|v| v.read)
    }

    pub(crate) fn cache_get(&mut self, key: &CacheKey<'grm, 'b>) -> Option<&PR> {
        if let Some(v) = self.cache.get_mut(key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    pub(crate) fn cache_insert(&mut self, key: CacheKey<'grm, 'b>, value: PR) {
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

pub fn parser_cache_recurse<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    sub: &'a impl Parser<'b, 'grm, PR<'grm>, E>,
    id: (
        ByAddress<&'b [BlockState<'b, 'grm>]>,
        ParserContext<'b, 'grm>,
    ),
) -> impl Parser<'b, 'grm, PR<'grm>, E> + 'a {
    move |pos_start: StringStream<'grm>,
          state: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>| {
        //Check if this result is cached
        let key = (pos_start.pos(), id.clone());
        if let Some(cached) = state.cache_get(&key) {
            return cached.clone();
        }

        //Before executing, put a value for the current position in the cache.
        //This value is used if the rule is left-recursive
        let mut res_recursive = PResult::new_err(E::new(pos_start.span_to(pos_start)), pos_start);
        res_recursive.add_label_explicit(Debug(pos_start.span_to(pos_start), "LEFTREC"));

        let cache_state = state.cache_state_get();
        state.cache_insert(key.clone(), res_recursive);

        //Now execute the parser_sugar rule, taking into account left recursion
        //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
        //A quick summary
        //- First put an error value for the current (rule, position) in the cache (already done)
        //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
        //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
        //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
        let res = sub.parse(pos_start, state, context);
        match res {
            POk(mut o, mut pos, mut be) => {
                //Did our rule left-recurse? (Safety: We just inserted it)
                if !state.cache_is_read(&key).unwrap() {
                    //No leftrec, cache and return
                    let res = POk(o, pos, be);
                    state.cache_insert(key, res.clone());
                    res
                } else {
                    //There was leftrec, we need to grow the seed
                    loop {
                        //Insert the current seed into the cache
                        state.cache_state_revert(cache_state);
                        state.cache_insert(key.clone(), POk(o.clone(), pos, be.clone()));

                        //Grow the seed
                        let new_res = sub.parse(pos_start, state, context);
                        match new_res {
                            POk(new_o, new_pos, new_be) if new_pos.cmp(pos).is_gt() => {
                                o = new_o;
                                pos = new_pos;
                                be = new_be;
                            }
                            POk(_, _, new_be) => {
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
                    POk(o, pos, be)
                }
            }
            res @ PErr(_, _) => {
                state.cache_insert(key, res.clone());
                res
            }
        }
    }
}
