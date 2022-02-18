use std::str::FromStr;

use dec::{Context, Decimal, Decimal128, Rounding};

pub fn with_digits(mut dec: Decimal128, digits: u32) -> Decimal128 {
    let mut ctx = Context::<Decimal128>::default();
    ctx.set_rounding(Rounding::HalfUp);
    
    dec = with_min_digits(&mut ctx, dec, digits);
    dec = with_max_digits(&mut ctx, dec, digits);

    dec
}

pub fn with_min_digits(ctx: &mut Context<Decimal128>, mut dec: Decimal128, digits: u32) -> Decimal128 {
    while dec.digits() < digits {
        let exp = dec.exponent();
        ctx.rescale(&mut dec, exp - 1)
    }

    dec
}

pub fn with_max_digits(ctx: &mut Context<Decimal128>, mut dec: Decimal128, digits: u32) -> Decimal128 {
    while dec.digits() > digits {
        let exp = dec.exponent();
        ctx.rescale(&mut dec, exp + 1)
    }

    dec
}

pub fn sqrt(dec: Decimal128) -> Decimal128 {
    let mut ctx = Context::<Decimal<12>>::default();
    ctx.set_rounding(Rounding::HalfUp);

    let mut dec: Decimal<12> = dec.into();
    ctx.sqrt::<12>(&mut dec);
    Decimal128::from_str(&dec.to_string()).unwrap()
}
