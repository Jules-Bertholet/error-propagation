use std::{
    cmp::min,
    fmt::Display,
    iter::{Product, Sum},
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
};

use dec::{Context, Decimal128, Rounding};

mod decimal;

#[derive(Clone, Copy, Debug, Default)]
pub struct UncertainDecimal {
    pub value: Decimal128,
    pub uncertainty: Decimal128,
}

impl Display for UncertainDecimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ± {}", self.value, self.uncertainty)
    }
}

impl UncertainDecimal {
    pub fn canonical(mut self) -> Self {
        let mut ctx = Context::<Decimal128>::default();
        ctx.set_rounding(Rounding::HalfUp);
        self.uncertainty = decimal::with_max_digits(&mut ctx, self.uncertainty.canonical(), 1);
        if self.value.exponent() <= self.uncertainty.exponent() {
            self.value = ctx.quantize(self.value, self.uncertainty);
        } else {
            self.uncertainty = Decimal128::ONE;
            ctx.set_exponent(&mut self.uncertainty, self.value.exponent());
        };

        self
    }

    pub fn with_digits(mut self, digits: u32) -> UncertainDecimal {
        self.value = decimal::with_digits(self.value, digits);

        self.canonical()
    }
}

impl Add for UncertainDecimal {
    type Output = UncertainDecimal;

    fn add(self, rhs: Self) -> Self::Output {
        UncertainDecimal {
            value: decimal::with_digits(
                self.value + rhs.value,
                min(self.value.digits(), rhs.value.digits()),
            ),
            uncertainty: decimal::sqrt(
                self.uncertainty * self.uncertainty + rhs.uncertainty * rhs.uncertainty,
            ),
        }
        .canonical()
    }
}

impl Div for UncertainDecimal {
    type Output = UncertainDecimal;

    fn div(self, rhs: Self) -> Self::Output {
        UncertainDecimal {
            value: decimal::with_digits(
                self.value / rhs.value,
                min(self.value.digits(), rhs.value.digits()),
            ),
            uncertainty: decimal::sqrt(
                self.uncertainty * self.uncertainty / self.value / self.value
                    + rhs.uncertainty * rhs.uncertainty / rhs.value / rhs.value,
            ) * self.value
                / rhs.value,
        }
        .canonical()
    }
}

impl Mul for UncertainDecimal {
    type Output = UncertainDecimal;

    fn mul(self, rhs: Self) -> Self::Output {
        UncertainDecimal {
            value: decimal::with_digits(
                self.value * rhs.value,
                min(self.value.digits(), rhs.value.digits()),
            ),
            uncertainty: decimal::sqrt(
                self.uncertainty * self.uncertainty / self.value / self.value
                    + rhs.uncertainty * rhs.uncertainty / rhs.value / rhs.value,
            ) * self.value
                * rhs.value,
        }
        .canonical()
    }
}

impl Neg for UncertainDecimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            value: -self.value,
            uncertainty: self.uncertainty,
        }
    }
}

impl Sub for UncertainDecimal {
    type Output = UncertainDecimal;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Product for UncertainDecimal {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut prod_v = Decimal128::ONE;
        let mut sum_sq_u = Decimal128::ONE;

        for UncertainDecimal { value, uncertainty } in iter {
            prod_v *= value;
            sum_sq_u += uncertainty * uncertainty / value / value;
        }

        UncertainDecimal {
            value: prod_v,
            uncertainty: decimal::sqrt(sum_sq_u) * prod_v,
        }
        .canonical()
    }
}

impl Sum for UncertainDecimal {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut sum_v = Decimal128::ZERO;
        let mut sum_sq_u = Decimal128::ZERO;

        for UncertainDecimal { value, uncertainty } in iter {
            sum_v += value;
            sum_sq_u += uncertainty * uncertainty;
        }

        UncertainDecimal {
            value: sum_v,
            uncertainty: decimal::sqrt(sum_sq_u),
        }
        .canonical()
    }
}

impl FromStr for UncertainDecimal {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (l, r) = s.split_once("±").ok_or(())?;

        Ok(Self {
            value: Decimal128::from_str(l.trim()).map_err(|_| ())?,
            uncertainty: Decimal128::from_str(r.trim()).map_err(|_| ())?,
        })
    }
}

pub fn average(decs: &[Decimal128]) -> UncertainDecimal {
    let len = Decimal128::from(decs.len() as u64);
    let avg: Decimal128 = decs.iter().sum::<Decimal128>() / len;

    let std_dev = decimal::sqrt(
        decs.iter()
            .map(|d| {
                let diff = *d - avg;
                diff * diff
            })
            .sum::<Decimal128>()
            / (len - Decimal128::ONE),
    );

    UncertainDecimal {
        value: avg,
        uncertainty: std_dev,
    }
}

#[macro_export]
macro_rules! ud {
    ($v:expr, $u:expr) => {
        (UncertainDecimal {
            value: Decimal128::from_str(stringify!($v)).unwrap(),
            uncertainty: Decimal128::from_str(stringify!($u)).unwrap(),
        })
    };
}

#[test]
fn test() {
    let a = ud!(1.7775, 0.6);

    println!("{}", a.canonical());

    let b = ud!(2000, 0.3).canonical();

    println!("{}", b);
    println!("{}", a + b);

    println!("{}", b.with_digits(8))
}
