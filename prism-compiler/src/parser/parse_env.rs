use crate::lang::{TcEnv, UnionIndex};
use prism_parser::core::cache::Allocs;
use prism_parser::core::input::Input;
use prism_parser::core::span::Span;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parsable::{Parsable2, ParseResult};

#[derive(Copy, Clone)]
pub struct ParsedEnv<'arn>(Option<&'arn ParsedEnvNode<'arn>>);

impl<'arn> ParsedEnv<'arn> {
    pub fn new_empty() -> Self {
        Self(None)
    }

    pub fn get(&self, name: &str) -> Option<(usize, &ParsedEnvNodeValue<'arn>)> {
        let mut current = self.0;
        let mut i = 0;
        while let Some(node) = current {
            if node.name == name {
                return Some((i, &node.value));
            }

            current = node.next;
            i += 1;
        }
        None
    }
}

#[derive(Copy, Clone)]
struct ParsedEnvNode<'arn> {
    name: &'arn str,
    next: Option<&'arn ParsedEnvNode<'arn>>,
    value: ParsedEnvNodeValue<'arn>,
}

#[derive(Copy, Clone)]
pub enum ParsedEnvNodeValue<'arn> {
    Substitute {
        subst: UnionIndex,
        subst_env: ParsedEnv<'arn>,
    },
    Type,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for ParsedEnv<'arn> {}
impl<'arn, 'grm: 'arn> Parsable2<'arn, 'grm, TcEnv> for ParsedEnv<'arn> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut TcEnv,
    ) -> Result<Self, String> {
        Ok(match constructor {
            "Nil" => ParsedEnv::new_empty(),
            "Substitute" => {
                assert_eq!(_args.len(), 4);
                let name = _args[0].into_value::<Input>().as_str(_src);
                let subst = *_args[1].into_value::<UnionIndex>();
                let subst_env = *_args[2].into_value::<ParsedEnv<'arn>>();
                let next = *_args[3].into_value::<ParsedEnv<'arn>>();
                ParsedEnv(Some(_allocs.alloc(ParsedEnvNode {
                    name,
                    next: next.0,
                    value: ParsedEnvNodeValue::Substitute { subst, subst_env },
                })))
            }
            "Type" => {
                assert_eq!(_args.len(), 2);

                let name = _args[0].into_value::<Input>().as_str(_src);
                let next = *_args[1].into_value::<ParsedEnv<'arn>>();
                ParsedEnv(Some(_allocs.alloc(ParsedEnvNode {
                    name,
                    next: next.0,
                    value: ParsedEnvNodeValue::Type,
                })))
            }
            _ => unreachable!(),
        })
    }
}
