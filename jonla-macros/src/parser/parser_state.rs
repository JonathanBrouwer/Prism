use std::collections::HashMap;
use crate::parser::core::error::{err_combine_opt, ParseError};
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult;
use crate::parser::core::presult::PResult::{PErr, POk};
use crate::parser::core::stream::Stream;
use crate::parser::parser_rule::PR;

pub struct ParserState<'grm, PR> {
    cache: HashMap<(usize, &'grm str), ParserCacheEntry<PR>>,
    cache_stack: Vec<(usize, &'grm str)>,
}

pub struct ParserCacheEntry<PR> {
    read: bool,
    value: PR,
}

impl<'grm, PR: Clone> ParserState<'grm, PR> {
    pub fn new() -> Self {
        ParserState {
            cache: HashMap::new(),
            cache_stack: Vec::new(),
        }
    }

    fn cache_is_read(&self, key: (usize, &'grm str)) -> Option<bool> {
        self.cache.get(&key).map(|v| v.read)
    }

    fn cache_get(&mut self, key: (usize, &'grm str)) -> Option<&PR> {
        if let Some(v) = self.cache.get_mut(&key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    fn cache_insert(&mut self, key: (usize, &'grm str), value: PR) {
        self.cache
            .insert(key, ParserCacheEntry { read: false, value });
        self.cache_stack.push(key);
    }

    fn cache_state_get(&self) -> usize {
        self.cache_stack.len()
    }

    fn cache_state_revert(&mut self, state: usize) {
        self.cache_stack.drain(state..).for_each(|key| {
            self.cache.remove(&key);
        })
    }
}



pub fn parser_cache_recurse<'grm: 'a, 'a, I: Clone + Eq, S: Stream<I = I>, E: ParseError + Clone>(
    sub: &'a impl Parser<I, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>>,
    id: &'grm str,
) -> impl Parser<I, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S, state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>| -> PResult<PR<'grm>, E, S> {
        //Check if this result is cached
        let key = (stream.pos(), id);
        if let Some(cached) = state.cache_get(key) {
            return cached.clone();
        }

        //Before executing, put a value for the current position in the cache.
        //This value is used if the rule is left-recursive
        let cache_state = state.cache_state_get();
        state.cache_insert(key, PResult::new_err(E::new(stream.span_to(stream)), stream));

        //Now execute the actual rule, taking into account left recursion
        //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
        //A quick summary
        //- First put an error value for the current (rule, position) in the cache (already done)
        //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
        //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
        //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
        let res = sub.parse(stream, state);
        match res {
            POk(mut o, mut pos, mut be) => {
                //Did our rule left-recurse? (Safety: We just inserted it)
                if !state.cache_is_read(key).unwrap() {
                    //No leftrec, cache and return
                    let res = POk(o, pos, be);
                    state.cache_insert(key, res.clone());
                    res
                } else {
                    //There was leftrec, we need to grow the seed
                    loop {
                        //Insert the current seed into the cache
                        state.cache_state_revert(cache_state);
                        state.cache_insert(key, POk(o.clone(), pos, be.clone()));

                        //Grow the seed
                        let new_res = sub.parse(pos, state);
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
            res@PErr(_, _) => {
                state.cache_insert(key, res.clone());
                res
            },
        }
    }
}
