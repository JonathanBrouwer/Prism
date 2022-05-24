#![allow(clippy::result_unit_err)]

use crate::core_parser::parse_error::{Expect, PEGParseError};
use crate::core_parser::parse_result::ParseResult;
use crate::core_parser::parser_core::{ParserContext, ParserState};
use crate::core_parser::parser_core_ast::{CoreExpression, ParsePairRaw};
use crate::core_parser::source_file::SourceFileIterator;
use crate::core_parser::span::Span;

/// Given an expression and the current position, attempts to parse this constructor.
pub fn parse_expression_name<'src>(
    state: &ParserContext<'src>,
    cache: &mut ParserState<'src>,
    expr_name: &'src str,
    pos: SourceFileIterator<'src>,
) -> ParseResult<'src, ParsePairRaw> {
    let expr: &'src CoreExpression = state
        .ast
        .sorts
        .get(expr_name)
        .expect("Name is guaranteed to exist");
    //Check if this result is cached
    let key = (pos.position(), expr_name);
    if let Some(cached) = cache.get_mut(&key) {
        return cached.clone();
    }

    //Before executing, put a value for the current position in the cache.
    //This value is used if the rule is left-recursive
    let cache_state = cache.state_current();
    cache.insert(
        key,
        ParseResult::new_err(
            ParsePairRaw::Error(Span::from_length(state.file, pos.position(), 0)),
            pos.clone(),
            pos.clone(),
        ),
    );

    //Now execute the actual rule, taking into account left recursion
    //The way this is done is heavily inspired by http://web.cs.ucla.edu/~todd/research/pepm08.pdf
    //A quick summary
    //- First put an error value for the current (rule, position) in the cache
    //- Try to parse the current (rule, position). If this fails, there is definitely no left recursion. Otherwise, we now have a seed.
    //- Put the new seed in the cache, and rerun on the current (rule, position). Make sure to revert the cache to the previous state.
    //- At some point, the above will fail. Either because no new input is parsed, or because the entire parse now failed. At this point, we have reached the maximum size.
    let mut res = parse_expression(state, cache, expr, pos.clone());
    let res = if res.ok {
        //Do we have a leftrec case?
        if !cache.is_read(&key).unwrap() {
            //There was no leftrec, just return the value
            res
        } else {
            //There was leftrec, we need to grow the seed
            loop {
                //Insert the current seed into the cache
                cache.state_revert(cache_state);
                cache.insert(key, res.clone());

                //Grow the seed
                let new_res = parse_expression(state, cache, expr, pos.clone());
                if !new_res.ok {
                    break;
                }
                if new_res.pos.position() <= res.pos.position() {
                    break;
                }
                res = new_res;
            }
            //The seed is at its maximum size
            cache.insert(key, res.clone());
            res
        }
    } else {
        // Left recursion value was used, but did not make a seed.
        // This is an illegal grammar!
        if cache.is_read(&key).unwrap() {
            cache.add_error(PEGParseError::fail_left_recursion(Span::from_length(
                state.file,
                pos.position(),
                0,
            )));
        }
        res
    };

    cache.insert(key, res.clone());

    //Return result
    res
}

pub fn parse_expression<'src>(
    state: &ParserContext<'src>,
    cache: &mut ParserState<'src>,
    expr: &'src CoreExpression,
    mut pos: SourceFileIterator<'src>,
) -> ParseResult<'src, ParsePairRaw> {
    match expr {
        //To parse a sort, call parse_sort recursively.
        CoreExpression::Name(sort_name) => {
            let res = parse_expression_name(state, cache, sort_name, pos);
            res.map(|s| ParsePairRaw::Name(s.span(), Box::new(s)))
        }
        //To parse a character class, check if the character is accepted, and make an ok/error based on that.
        CoreExpression::CharacterClass(characters) => {
            let span = Span::from_length(state.file, pos.position(), 1);
            if pos.accept(characters) {
                ParseResult::new_ok(ParsePairRaw::Empty(span), pos.clone(), pos, false)
            } else {
                cache.add_error(PEGParseError::expect(
                    span.clone(),
                    Expect::ExpectCharClass(characters.clone()),
                ));
                ParseResult::new_err(ParsePairRaw::Error(span), pos.clone(), pos)
            }
        }
        //To parse a sequence, parse each constructor in the sequence.
        //The results are added to `results`, and the best error and position are updated each time.
        //Finally, construct a `ParsePairConstructor::List` with the results.
        CoreExpression::Sequence(subexprs) => {
            let mut results = vec![];
            let start_pos = pos.position();
            let mut pos_err = pos.clone();
            let mut recovered = false;

            //Parse all subconstructors in sequence
            for (i, subexpr) in subexprs.iter().enumerate() {
                let res = parse_expression(state, cache, subexpr, pos);
                pos = res.pos;
                pos_err.max_pos(res.pos_err.clone());
                results.push(res.result);
                recovered |= res.recovered;
                if !res.ok {
                    if let Some(&offset) = state.errors.get(&res.pos_err.position()) {
                        //The first token of the sequence can not be skipped, otherwise we can just parse a lot of empty sequences, if a sequence happens in a repeat
                        if i != 0 && cache.no_errors_nest_count == 0 {
                            pos = res.pos_err;
                            //If we're at the end of the file, don't try
                            if pos.peek().is_none() {
                                let span = Span::from_end(state.file, start_pos, pos.position());
                                return ParseResult::new_err(
                                    ParsePairRaw::List(span, results),
                                    pos,
                                    pos_err,
                                );
                            }
                            pos.skip_n(offset);
                            recovered = true;
                            continue;
                        }
                    }

                    let start_pos = results
                        .get(0)
                        .map(|pp| pp.span().position)
                        .unwrap_or(start_pos);
                    let span = Span::from_end(state.file, start_pos, pos.position());
                    return ParseResult::new_err(ParsePairRaw::List(span, results), pos, pos_err);
                }
            }

            //Construct result
            let start_pos = results
                .get(0)
                .map(|pp| pp.span().position)
                .unwrap_or(start_pos);
            let span = Span::from_end(state.file, start_pos, pos.position());
            ParseResult::new_ok(ParsePairRaw::List(span, results), pos, pos_err, recovered)
        }
        //To parse a sequence, first parse the minimum amount that is needed.
        //Then keep trying to parse the constructor until the maximum is reached.
        //The results are added to `results`, and the best error and position are updated each time.
        //Finally, construct a `ParsePairConstructor::List` with the results.
        CoreExpression::Repeat { subexpr, min, max } => {
            let mut results = vec![];
            let start_pos = pos.position();
            let mut last_pos = pos.position();
            let mut pos_err = pos.clone();
            let mut recovered = false;

            //Parse at most maximum times
            for i in 0..max.unwrap_or(u64::MAX) {
                let res = parse_expression(state, cache, subexpr.as_ref(), pos.clone());
                pos_err.max_pos(res.pos_err.clone());
                recovered |= res.recovered;

                if res.ok {
                    pos = res.pos;
                    results.push(res.result);
                } else {
                    //If we know about this error, try to continue?
                    //Don't try to continue if we haven't made any progress (already failed on first character), since we will just fail again
                    //Also don't try to continue if we don't allow errors at the moment, since we don't want to try to recover inside of an no-errors segment
                    if let Some(&offset) = state.errors.get(&res.pos_err.position()) {
                        if (offset > 0 || pos.position() != res.pos_err.position())
                            && cache.no_errors_nest_count == 0
                        {
                            pos = res.pos_err.clone();
                            //If we're at the end of the file, don't try
                            if pos.peek().is_none() {
                                let span = Span::from_end(state.file, start_pos, pos.position());
                                return ParseResult::new_err(
                                    ParsePairRaw::List(span, results),
                                    pos,
                                    pos_err,
                                );
                            }
                            pos.skip_n(offset);
                            results.push(res.result);
                            recovered = true;
                            continue;
                        }
                    }
                    //If we have not yet reached the minimum, we error.
                    //Otherwise, we break and ok after the loop body.
                    //In case we reached the minimum, we don't push the error, even though the failure might've been an error.
                    //This is because it's probably OK, and we want no Error pairs in the parse tree when it's OK.
                    if i < *min {
                        pos = res.pos;
                        results.push(res.result);
                        let start_pos = results
                            .get(0)
                            .map(|pp| pp.span().position)
                            .unwrap_or(start_pos);
                        let span = Span::from_end(state.file, start_pos, pos.position());
                        return ParseResult::new_err(
                            ParsePairRaw::List(span, results),
                            pos,
                            pos_err,
                        );
                    } else {
                        break;
                    }
                }
                //If the position hasn't changed, then we're in an infinite loop
                if last_pos == pos.position() {
                    let span = Span::from_length(state.file, pos.position(), 0);
                    cache.add_error(PEGParseError::fail_loop(span.clone()));
                    return ParseResult::new_err(ParsePairRaw::List(span, results), pos, pos_err);
                }
                last_pos = pos.position();
            }

            //Construct result
            let start_pos = results
                .get(0)
                .map(|pp| pp.span().position)
                .unwrap_or(start_pos);
            let span = Span::from_end(state.file, start_pos, pos.position());
            ParseResult::new_ok(
                ParsePairRaw::List(span, results),
                pos.clone(),
                pos_err,
                recovered,
            )
        }
        //To parse a choice, try each constructor, keeping track of the best error that occurred while doing so.
        //If none of the constructors succeed, we will return this error.
        CoreExpression::Choice(subexprs) => {
            //Try each constructor, keeping track of the best error that occurred while doing so.
            //If none of the constructors succeed, we will return this error.
            let mut results = vec![];
            assert!(!subexprs.is_empty());
            for (i, subexpr) in subexprs.iter().enumerate() {
                let res = parse_expression(state, cache, subexpr, pos.clone());
                if res.ok && !res.recovered {
                    return ParseResult::new_ok(
                        ParsePairRaw::Choice(res.result.span(), i, Box::new(res.result)),
                        res.pos,
                        res.pos_err,
                        res.recovered,
                    );
                }
                results.push(res);
            }
            //Chose best candidate
            let (i, res) = results
                .into_iter()
                .enumerate()
                .max_by_key(|(_, r)| r.pos_err.position())
                .unwrap();
            ParseResult::new(
                ParsePairRaw::Choice(res.result.span(), i, Box::new(res.result)),
                res.pos,
                res.pos_err,
                res.ok,
                res.recovered,
            )
        }
        //No errors is parsed by setting the no errors flag during parsing
        //After the block is completed, is not ok, produce an error.
        CoreExpression::FlagNoErrors(subexpr, name) => {
            cache.no_errors_nest_count += 1;
            let start_pos = pos.position();
            let res = parse_expression(state, cache, subexpr, pos.clone());
            cache.no_errors_nest_count -= 1;
            if !res.ok {
                let mut next_pos = res.pos.clone();
                next_pos.skip_n(1);
                let span = Span::from_end(state.file, start_pos, next_pos.position());
                let err = PEGParseError::expect(span, Expect::ExpectSort(name.to_string()));
                cache.add_error(err);
            }
            res
        }
    }
}