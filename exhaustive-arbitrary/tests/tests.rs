use exhaustive_arbitrary::ExhaustiveArbitrary;

#[test]
fn test_bool() {
    assert_eq!(
        bool::iter_exhaustive(0).collect::<Vec<_>>(),
        vec![false, true],
    );
}

#[test]
fn test_vec_bool() {
    assert_eq!(
        Vec::<bool>::iter_exhaustive(2).collect::<Vec<_>>(),
        vec![
            vec![],
            vec![false],
            vec![true],
            vec![false, false],
            vec![false, true],
            vec![true, false],
            vec![true, true],
        ],
    );
}



// #[test]
// fn test_reverse() {
//     for v in Vec::<bool>::iter_exhaustive(8) {
//
//     }
// }