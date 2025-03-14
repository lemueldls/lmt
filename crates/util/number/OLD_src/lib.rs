// mod decimal;
mod integer;

// pub use decimal::LmtDecimal;
pub use integer::LmtInteger;
pub type LmtDecimal = num_rational::Ratio<LmtInteger>;
