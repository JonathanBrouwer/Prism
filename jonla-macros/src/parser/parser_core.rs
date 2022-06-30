use crate::grammar::CharClass;
use crate::parser::parser_result::ParseErrorLabel::RemainingInputNotParsed;
use crate::parser::parser_result::{ParseError, ParseErrorLabel, ParseResult};
use std::collections::HashMap;

pub struct ParserState<'grm, 'src, CT: Clone> {
    input: &'src str,

    cache: HashMap<(usize, &'grm str), ParserCacheEntry<'grm, CT>>,
    cache_stack: Vec<(usize, &'grm str)>,
}

pub struct ParserCacheEntry<'grm, CT: Clone> {
    read: bool,
    value: ParseResult<'grm, CT>,
}

impl<'grm, 'src, CT: Clone> ParserState<'grm, 'src, CT> {
    pub fn new(input: &'src str) -> Self {
        ParserState {
            input,
            cache: HashMap::new(),
            cache_stack: Vec::new(),
        }
    }

    pub fn parse_charclass(&mut self, pos: usize, cc: &CharClass) -> ParseResult<'grm, ()> {
        match self.input[pos..].chars().next() {
            Some(c) if cc.contains(c) => ParseResult::new_ok((), pos + c.len_utf8()),
            _ => ParseResult::new_err(pos, vec![ParseErrorLabel::CharClass(cc.clone())]),
        }
    }

    ///
    /// Recovery:
    /// - If left matched zero input, don't bother continuing, we'll succeed higher up
    /// - If left matched some amount, try to continue and match more
    pub fn parse_sequence<S: Clone, T: Clone>(
        &mut self,
        res_left: ParseResult<'grm, S>,
        right: impl Fn(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<'grm, T>,
    ) -> ParseResult<'grm, (S, T)> {
        match res_left.inner {
            Ok(ok_left) => {
                let res_right: ParseResult<T> = right(self, ok_left.pos);
                match res_right.inner {
                    Ok(ok_right) => ParseResult::new_ok_with_err(
                        (ok_left.result, ok_right.result),
                        ok_right.pos,
                        ok_right.best_error,
                    ),
                    Err(err_right) => ParseResult::from_err(
                        ParseError::combine_option_parse_error(ok_left.best_error, Some(err_right))
                            .unwrap(),
                    ),
                }
            }
            Err(err_left) => ParseResult::from_err(err_left),
        }
    }

    pub fn parse_choice<T: Clone>(
        &mut self,
        pos: usize,
        res_left: ParseResult<'grm, T>,
        right: impl FnOnce(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<'grm, T>,
    ) -> ParseResult<'grm, T> {
        match res_left.inner {
            Ok(ok_left) => {
                //TODO in case left terminated early, we should run right side and add to best error
                ParseResult::from_ok(ok_left)
            }
            Err(err_left) => {
                let res_right: ParseResult<T> = right(self, pos);
                match res_right.inner {
                    Ok(mut ok_right) => {
                        ok_right.best_error = ParseError::combine_option_parse_error(
                            Some(err_left),
                            ok_right.best_error,
                        );
                        ParseResult::from_ok(ok_right)
                    }
                    Err(err_right) => ParseResult::from_err(err_left.combine(err_right)),
                }
            }
        }
    }

    fn cache_is_read(&self, key: (usize, &'grm str)) -> Option<bool> {
        self.cache.get(&key).map(|v| v.read)
    }

    fn cache_get(&mut self, key: (usize, &'grm str)) -> Option<&ParseResult<'grm, CT>> {
        if let Some(v) = self.cache.get_mut(&key) {
            v.read = true;
            Some(&v.value)
        } else {
            None
        }
    }

    fn cache_insert(&mut self, key: (usize, &'grm str), value: ParseResult<'grm, CT>) {
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
        sub: impl Fn(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<'grm, CT>,
        id: &'grm str,
    ) -> ParseResult<'grm, CT> {
        //Check if this result is cached
        let key = (pos, id);
        if let Some(cached) = self.cache_get(key) {
            return cached.clone();
        }

        //Before executing, put a value for the current position in the cache.
        //This value is used if the rule is left-recursive
        let cache_state = self.cache_state_get();
        self.cache_insert(key, ParseResult::new_err_leftrec(pos));

        //Now execute the actual rule, taking into account left recursion
        //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
        //A quick summary
        //- First put an error value for the current (rule, position) in the cache (already done)
        //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
        //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
        //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
        let res = sub(self, pos);
        match res.inner {
            Ok(mut ok) => {
                //Did our rule left-recurse? (Safety: We just inserted it)
                if !self.cache_is_read(key).unwrap() {
                    //No leftrec, cache and return
                    let res = ParseResult::from_ok(ok);
                    self.cache_insert(key, res.clone());
                    res
                } else {
                    //There was leftrec, we need to grow the seed
                    loop {
                        //Insert the current seed into the cache
                        self.cache_state_revert(cache_state);
                        self.cache_insert(key, ParseResult::from_ok(ok.clone()));

                        //Grow the seed
                        let new_res = sub(self, pos);
                        match new_res.inner {
                            Ok(new_ok) if new_ok.pos > ok.pos => {
                                ok = new_ok;
                            }
                            _ => {
                                break;
                            }
                        }
                    }

                    //The seed is at its maximum size
                    //It should still be in the cache,
                    ParseResult::from_ok(ok)
                }
            }
            Err(err) => {
                // Left recursion value was used, but did not make a seed.
                // This is an illegal grammar!
                if self.cache_is_read(key).unwrap() {
                    return ParseResult::new_err_leftrec(pos);
                } else {
                    //Not ok, but seed was not used. This is just normal error.
                    //Insert into cache then return
                    let res = ParseResult::from_err(err);
                    self.cache_insert(key, res.clone());
                    res
                }
            }
        }
    }

    pub fn parse_full_input<T: Clone>(
        &mut self,
        sub: impl Fn(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<'grm, T>,
    ) -> ParseResult<'grm, T> {
        let res = sub(self, 0);
        match res.inner {
            Ok(ok) if res.pos() == self.input.len() => ParseResult::from_ok(ok),
            Ok(ok) => ok
                .best_error
                .map(ParseResult::from_err)
                .unwrap_or(ParseResult::new_err(ok.pos, vec![RemainingInputNotParsed])),
            Err(err) => ParseResult::from_err(err),
        }
    }
}
