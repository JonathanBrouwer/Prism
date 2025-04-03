use crate::core::adaptive::{BlockState, GrammarStateId};
use crate::core::arc_ref::BorrowedArcSlice;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::{ParseError, err_combine_opt};
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct CacheKey {
    pos: Pos,
    block: usize,     // Start of blocks ptr to usize
    rule_args: usize, //TODO
    ctx: ParserContext,
    state: GrammarStateId,
    eval_ctx: usize,
}
pub type CacheVal<E> = PResult<Parsed, E>;

pub struct ParserCacheEntry<PR> {
    pub read: bool,
    pub value: PR,
}

impl<Db, E: ParseError<L = ErrorLabel>> ParserState<Db, E> {
    pub fn parse_cache_recurse(
        &mut self,
        mut sub: impl FnMut(&mut ParserState<Db, E>, Pos) -> PResult<Parsed, E>,
        blocks: BorrowedArcSlice<Arc<BlockState>>,
        rule_args: &VarMap,
        grammar_state: GrammarStateId,
        pos_start: Pos,
        context: ParserContext,
    ) -> PResult<Parsed, E> {
        //Check if this result is cached
        let mut args_hash = DefaultHasher::new();
        for (name, value) in rule_args.iter() {
            args_hash.write(name.as_str().as_bytes());
            args_hash.write_usize(value.as_ptr().as_ptr() as usize);
        }

        let key = CacheKey {
            pos: pos_start,
            block: blocks.as_ptr() as usize,
            rule_args: args_hash.finish() as usize,
            ctx: context,
            state: grammar_state,
            eval_ctx: 0, //TODO insert eval ctx as part of cache entry
        };
        if let Some(cached) = self.cache_get(&key) {
            return cached.clone();
        }

        //Before executing, put a value for the current position in the cache.
        //This value is used if the rule is left-recursive
        let mut res_recursive = PResult::new_err(E::new(pos_start), pos_start);
        res_recursive
            .add_label_explicit(Debug(pos_start.span_to(pos_start), "LEFTREC".to_string()));

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
            POk {
                obj: mut o,
                start: mut spos,
                end: mut epos,
                best_err: mut be,
            } => {
                //Did our rule left-recurse? (Safety: We just inserted it)
                if !self.cache_is_read(key.clone()).unwrap() {
                    //No leftrec, cache and return
                    let res = POk {
                        obj: o,
                        start: spos,
                        end: epos,
                        best_err: be,
                    };
                    self.cache_insert(key, res.clone());
                    res
                } else {
                    //There was leftrec, we need to grow the seed
                    loop {
                        //Insert the current seed into the cache
                        self.cache_state_revert(cache_state);
                        self.cache_insert(
                            key.clone(),
                            POk {
                                obj: o.clone(),
                                start: spos,
                                end: epos,
                                best_err: be.clone(),
                            },
                        );

                        //Grow the seed
                        let new_res = sub(self, pos_start);
                        match new_res {
                            POk {
                                obj: new_o,
                                start: new_spos,
                                end: new_epos,
                                best_err: new_be,
                            } if new_epos.cmp(&epos).is_gt() => {
                                o = new_o;
                                spos = new_spos;
                                epos = new_epos;
                                be = new_be;
                            }
                            POk {
                                obj: _,
                                start: _,
                                end: _,
                                best_err: new_be,
                            } => {
                                be = err_combine_opt(be, new_be);
                                break;
                            }
                            PErr {
                                err: new_e,
                                end: new_s,
                            } => {
                                be = err_combine_opt(be, Some((new_e, new_s)));
                                break;
                            }
                        }
                    }

                    //The seed is at its maximum size
                    //It should still be in the cache,
                    POk {
                        obj: o,
                        start: spos,
                        end: epos,
                        best_err: be,
                    }
                }
            }
            res @ PErr { err: _, end: _ } => {
                self.cache_insert(key, res.clone());
                res
            }
        }
    }
}
