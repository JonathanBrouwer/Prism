use crate::{ParseError, Parser, ParseSuccess};
use crate::peg::input::Input;
use crate::peg::parsers::ignore_whitespace::ignore_whitespace;

macro_rules! generate_seq (
    ($fname:ident $($vr:ident $ok:ident $tp:ident)*) => {
        #[allow(unused_parens)]
        #[allow(unused_mut)]
        pub fn $fname<I: Input<InputElement=IE>, IE, $($tp),*>($($vr: impl Parser<I, $tp>),*)
            -> impl Parser<I, ($($tp),*)> {
            move |mut pos: I| {
                let mut best_error: Option<ParseError<I>> = None;

                generate_seq!(__inner pos best_error : $($vr $ok)*);

                Ok(ParseSuccess {
                  result: ($($ok.result),*),
                  best_error,
                  pos
                })
            }
        }
    };
    (__inner $pos:ident $best_error:ident : ) => {};
    (__inner $pos:ident $best_error:ident : $vr0:ident $ok0:ident $($vr:ident $ok:ident)*) => {
        let $ok0 = $vr0.parse($pos)?;
        $pos = $ok0.pos;
        $best_error = ParseError::parse_error_combine_opt2($best_error, $ok0.best_error);

        generate_seq!(__inner $pos $best_error : $($vr $ok)*);
    }
);

macro_rules! generate_seq_ws {
    ($fname:ident $fname2:ident ::: $($vr: ident $tp: ident)*) => {
        #[allow(unused_parens)]
        #[allow(unused_mut)]
        pub fn $fname2<I: Input<InputElement=char>, $($tp),*>($($vr: impl Parser<I, $tp>),*)
            -> impl Parser<I, ($($tp),*)> {
            $fname($(ignore_whitespace($vr)),*)
        }
    }
}

macro_rules! generate_seqs {
    ($fname:ident ::: $fname2:ident ::: ) => {
        generate_seq!{$fname }
        generate_seq_ws!{$fname $fname2 ::: }
    };
    ($fname:ident $($fnames:ident)* ::: $fname2:ident $($fnames2:ident)* ::: $v0:ident $ok0:ident $O0:ident $($vr:ident $ok:ident $tp:ident)*) => {
        generate_seq!{$fname $v0 $ok0 $O0 $($vr $ok $tp)*}
        generate_seq_ws!{$fname $fname2 ::: $v0 $O0 $($vr $tp)* }
        generate_seqs!{$($fnames)* ::: $($fnames2)* ::: $($vr $ok $tp)*}
    }
}

generate_seqs!{
            seq20 seq19 seq18 seq17 seq16 seq15 seq14 seq13 seq12 seq11 seq10 seq9 seq8 seq7 seq6 seq5 seq4 seq3 seq2 seq1 seq0 :::
            seq20ws seq19ws seq18ws seq17ws seq16ws seq15ws seq14ws seq13ws seq12ws seq11ws seq10ws seq9ws seq8ws seq7ws seq6ws seq5ws seq4ws seq3ws seq2ws seq1ws seq0ws :::
            v1 ok1 O1 v2 ok2 O2 v3 ok3 O3 v4 ok4 O4 v5 ok5 O5 v6 ok6 O6 v7 ok7 O7
            v8 ok8 O8 v9 ok9 O9 v10 ok10 O10 v11 ok11 O11 v12 ok12 O12 v13 ok13 O13
            v14 ok14 O14 v15 ok15 O15 v16 ok16 O16 v17 ok17 O17 v18 ok18 O18 v19 ok19 O19 v20 ok20 O20}