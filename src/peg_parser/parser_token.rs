use std::hash::Hash;

pub trait Token<TT: TokenType, TV: TokenValue>: Sized + Clone + Eq + Hash {
    fn to_val(&self) -> TV;
    fn to_type(&self) -> TT;
}

pub trait TokenValue: Sized + Copy + Eq + Hash {

}

pub trait TokenType: Sized + Copy + Eq + Hash {

}
