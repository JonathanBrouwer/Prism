use std::collections::HashMap;
use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult;
use crate::parser::core::presult::PResult::POk;
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



// pub fn parser_cache_recurse<'a, 'grm, I: Clone + Eq, S: Stream<I = I>, E: ParseError + Clone>(
//     sub: &'a impl Parser<I, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>>,
//     id: &'grm str,
// ) -> impl Parser<I, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> + 'a {
//     move |stream: S, state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>| -> PResult<PR<'grm>, E, S> {
//         //Check if this result is cached
//         let key = (stream.pos(), id);
//         if let Some(cached) = state.cache_get(key) {
//             return cached.clone();
//         }
//
//         //Before executing, put a value for the current position in the cache.
//         //This value is used if the rule is left-recursive
//         let cache_state = state.cache_state_get();
//         state.cache_insert(key, PResult::new_err(E::new(stream.span_to(stream)), stream));
//
//         //Now execute the actual rule, taking into account left recursion
//         //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
//         //A quick summary
//         //- First put an error value for the current (rule, position) in the cache (already done)
//         //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
//         //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
//         //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
//         let res = sub.parse(stream, state);
//         match res {
//             /*
//             Ok(mut ok) => {
//                 //Did our rule left-recurse? (Safety: We just inserted it)
//                 if !state.cache_is_read(key).unwrap() {
//                     //No leftrec, cache and return
//                     let res = ParseResult::from_ok(ok);
//                     state.cache_insert(key, res.clone());
//                     res
//                 } else {
//                     //There was leftrec, we need to grow the seed
//                     loop {
//                         //Insert the current seed into the cache
//                         state.cache_state_revert(cache_state);
//                         state.cache_insert(key, ParseResult::from_ok(ok.clone()));
//
//                         //Grow the seed
//                         let new_res = sub(self, pos);
//                         match new_res.inner {
//                             Ok(new_ok) if new_ok.pos > ok.pos => {
//                                 ok = new_ok;
//                             }
//                             _ => {
//                                 break;
//                             }
//                         }
//                     }
//
//                     //The seed is at its maximum size
//                     //It should still be in the cache,
//                     ParseResult::from_ok(ok)
//                 }
//             }
//             Err(err) => {
//                 // Left recursion value was used, but did not make a seed.
//                 // This is an illegal grammar!
//                 if state.cache_is_read(key).unwrap() {
//                     return ParseResult::new_err_leftrec(pos);
//                 } else {
//                     //Not ok, but seed was not used. This is just normal error.
//                     //Insert into cache then return
//                     let res = ParseResult::from_err(err);
//                     state.cache_insert(key, res.clone());
//                     res
//                 }*/
//             POk(o, s) => {
//                 //Did our rule left-recurse? (Safety: We just inserted it)
//                 if !state.cache_is_read(key).unwrap() {
//                     //No leftrec, cache and return
//                     let res = POk(o, s);
//                     state.cache_insert(key, res.clone());
//                     res
//                 } else {
//                     //There was leftrec, we need to grow the seed
//                     loop {
//                         //Insert the current seed into the cache
//                         state.cache_state_revert(cache_state);
//                         state.cache_insert(key, POk(o.clone(), s));
//
//                         //Grow the seed
//                         let new_res = sub.parse(stream, state);
//                         match new_res.inner {
//                             Ok(new_ok) if new_ok.pos > ok.pos => {
//                                 ok = new_ok;
//                             }
//                             _ => {
//                                 break;
//                             }
//                         }
//                     }
//
//                     //The seed is at its maximum size
//                     //It should still be in the cache,
//                     ParseResult::from_ok(ok)
//                 }
//             }
//             PResult::PRec(_, _, _) => todo!(),
//             PResult::PErr(_, _, _) => todo!(),
//         }
//     }
// }
