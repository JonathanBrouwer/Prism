use crate::grammar::CharClass;
use crate::parser::parser_result::ParseResult;
use std::collections::{HashMap};

pub struct ParserState<'grm, 'src, CT: Clone> {
    input: &'src str,

    cache: HashMap<(usize, &'grm str), ParserCacheEntry<CT>>,
    cache_stack: Vec<(usize, &'grm str)>,
}

pub struct ParserCacheEntry<CT: Clone> {
    read: bool,
    value: ParseResult<CT>,
}

impl<'grm, 'src, CT: Clone> ParserState<'grm, 'src, CT> {
    pub fn new(input: &'src str) -> Self {
        ParserState {
            input,
            cache: HashMap::new(),
            cache_stack: Vec::new(),
        }
    }

    pub fn parse_charclass(&mut self, pos: usize, cc: &CharClass) -> ParseResult<()> {
        match self.input[pos..].chars().next() {
            Some(c) if cc.contains(c) => ParseResult::new_ok((), pos + c.len_utf8()),
            _ => ParseResult::new_err(pos),
        }
    }

    ///
    /// Recovery:
    /// - If left matched zero input, don't bother continuing, we'll succeed higher up
    /// - If left matched some amount, try to continue and match more
    pub fn parse_sequence<S: Clone, T: Clone>(
        &mut self,
        res_left: ParseResult<S>,
        right: impl Fn(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<T>,
    ) -> ParseResult<(S, T)> {
        match res_left.result {
            Some(s) => {
                let res_right: ParseResult<T> = right(self, res_left.pos);
                if res_right.is_ok() {
                    ParseResult::new_ok((s, res_right.result.unwrap()), res_right.pos)
                } else {
                    ParseResult::new_err(res_right.pos)
                }
            }
            None => res_left.map(|_| unreachable!()),
        }
    }

    pub fn parse_choice<T: Clone>(
        &mut self,
        pos: usize,
        res_left: ParseResult<T>,
        right: impl Fn(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<T>,
    ) -> ParseResult<T> {
        match res_left.result {
            Some(_) => res_left,
            None => {
                let mut res_right: ParseResult<T> = right(self, pos);
                if res_right.is_ok() {
                    res_right
                } else {
                    res_right.pos = res_left.pos.max(res_right.pos);
                    res_right
                }
            }
        }
    }

    fn cache_is_read(&self, key: (usize, &'grm str)) -> Option<bool> {
        self.cache.get(&key).map(|v| v.read)
    }

    fn cache_get(&mut self, key: (usize, &'grm str)) -> Option<&ParseResult<CT>> {
        if let Some(v) = self.cache.get_mut(&key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    fn cache_insert(&mut self, key: (usize, &'grm str), value: ParseResult<CT>) {
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

    pub fn parse_cache_recurse(
        &mut self,
        pos: usize,
        sub: impl Fn(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<CT>,
        id: &'grm str,
    ) -> ParseResult<CT> {
        //Check if this result is cached
        let key = (pos, id);
        if let Some(cached) = self.cache_get(key) {
            return cached.clone();
        }

        //Before executing, put a value for the current position in the cache.
        //This value is used if the rule is left-recursive
        let cache_state = self.cache_state_get();
        self.cache_insert(key, ParseResult::new_err(pos));

        //Now execute the actual rule, taking into account left recursion
        //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
        //A quick summary
        //- First put an error value for the current (rule, position) in the cache (already done)
        //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
        //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
        //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
        let mut res = sub(self, pos);
        if res.is_ok() {
            //Did our rule left-recurse? (Safety: We just inserted it)
            if !self.cache_is_read(key).unwrap() {
                //No leftrec, cache and return
                self.cache_insert(key, res.clone());
                res
            } else {
                //There was leftrec, we need to grow the seed
                loop {
                    //Insert the current seed into the cache
                    self.cache_state_revert(cache_state);
                    self.cache_insert(key, res.clone());

                    //Grow the seed
                    let new_res = sub(self, pos);
                    if !new_res.is_ok() {
                        break;
                    }
                    if new_res.pos <= res.pos {
                        break;
                    }
                    res = new_res;
                }

                //The seed is at its maximum size
                //It should still be in the cache,
                res
            }
        } else {
            // Left recursion value was used, but did not make a seed.
            // This is an illegal grammar!
            if self.cache_is_read(key).unwrap() {
                return ParseResult::new_err(pos);
            } else {
                //Not ok, but seed was not used. This is just normal error.
                //Insert into cache then return
                self.cache_insert(key, res.clone());
                res
            }
        }
    }

    pub fn parse_full_input<T: Clone>(
        &mut self,
        sub: impl Fn(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<T>,
    ) -> ParseResult<T> {
        let res = sub(self, 0);
        if !res.is_ok() {
            return res
        }

        if res.pos == self.input.len() {
            res
        } else {
            //TODO find best error
            ParseResult::new_err(res.pos)
        }
    }


}
