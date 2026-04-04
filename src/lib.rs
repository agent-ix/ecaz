use pgrx::prelude::*;

pgrx::pg_module_magic!();

/// A fixed-point quantitative value backed by a 64-bit integer.
/// Stores values as integer * 10^(-scale) for lossless decimal arithmetic.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord,
    PostgresType, PostgresEq, PostgresOrd,
)]
#[inoutfuncs]
pub struct TurboQuant {
    /// Raw integer mantissa
    value: i64,
    /// Decimal scale (number of digits after the decimal point)
    scale: i16,
}

impl InOutFuncs for TurboQuant {
    fn input(input: &core::ffi::CStr) -> Self
    where
        Self: Sized,
    {
        let s = input.to_str().expect("invalid UTF-8 in turboquant input");
        TurboQuant::parse(s).unwrap_or_else(|e| panic!("invalid turboquant: {e}"))
    }

    fn output(&self, buffer: &mut pgrx::StringInfo) {
        buffer.push_str(&self.to_display_string());
    }
}

impl TurboQuant {
    pub fn new(value: i64, scale: i16) -> Self {
        Self { value, scale }
    }

    /// Parse a decimal string like "123.456" into a TurboQuant.
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        match s.find('.') {
            None => {
                let value: i64 = s.parse().map_err(|e| format!("parse error: {e}"))?;
                Ok(Self { value, scale: 0 })
            }
            Some(dot_pos) => {
                let frac = &s[dot_pos + 1..];
                let scale = frac.len() as i16;
                let combined = format!("{}{}", &s[..dot_pos], frac);
                let value: i64 = combined.parse().map_err(|e| format!("parse error: {e}"))?;
                Ok(Self { value, scale })
            }
        }
    }

    pub fn to_display_string(&self) -> String {
        if self.scale == 0 {
            return self.value.to_string();
        }
        let s = format!("{:0>width$}", self.value.abs(), width = self.scale as usize + 1);
        let split = s.len() - self.scale as usize;
        let sign = if self.value < 0 { "-" } else { "" };
        format!("{}{}.{}", sign, &s[..split], &s[split..])
    }

    /// Rescale to a common scale for arithmetic.
    fn rescale(a: Self, b: Self) -> (Self, Self) {
        if a.scale == b.scale {
            return (a, b);
        }
        if a.scale < b.scale {
            let diff = (b.scale - a.scale) as u32;
            let factor = 10i64.pow(diff);
            (Self { value: a.value * factor, scale: b.scale }, b)
        } else {
            let diff = (a.scale - b.scale) as u32;
            let factor = 10i64.pow(diff);
            (a, Self { value: b.value * factor, scale: a.scale })
        }
    }
}

// --- Arithmetic operators ---

#[pg_operator(immutable, parallel_safe)]
#[opname(+)]
fn turboquant_add(a: TurboQuant, b: TurboQuant) -> TurboQuant {
    let (a, b) = TurboQuant::rescale(a, b);
    TurboQuant { value: a.value + b.value, scale: a.scale }
}

#[pg_operator(immutable, parallel_safe)]
#[opname(-)]
fn turboquant_sub(a: TurboQuant, b: TurboQuant) -> TurboQuant {
    let (a, b) = TurboQuant::rescale(a, b);
    TurboQuant { value: a.value - b.value, scale: a.scale }
}

#[pg_operator(immutable, parallel_safe)]
#[opname(*)]
fn turboquant_mul(a: TurboQuant, b: TurboQuant) -> TurboQuant {
    TurboQuant {
        value: a.value * b.value,
        scale: a.scale + b.scale,
    }
}

// --- Aggregate: sum ---

#[pg_aggregate]
impl Aggregate for TurboQuant {
    type State = Option<TurboQuant>;
    const NAME: &'static str = "turboquant_sum";

    #[pgrx(parallel_safe, immutable)]
    fn state(
        mut current: Self::State,
        v: Self,
        _fcinfo: pg_sys::FunctionCallInfo,
    ) -> Self::State {
        Some(match current.take() {
            None => v,
            Some(acc) => turboquant_add(acc, v),
        })
    }
}

// --- Casts ---

#[pg_extern(immutable, parallel_safe)]
fn turboquant_from_float8(v: f64) -> TurboQuant {
    TurboQuant::parse(&format!("{:.10}", v))
        .unwrap_or_else(|e| panic!("cast error: {e}"))
}

#[pg_extern(immutable, parallel_safe)]
fn turboquant_to_float8(v: TurboQuant) -> f64 {
    v.to_display_string().parse().unwrap()
}

// --- Tests ---

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use pgrx::prelude::*;

    #[pg_test]
    fn test_parse_integer() {
        let q = TurboQuant::parse("42").unwrap();
        assert_eq!(q.value, 42);
        assert_eq!(q.scale, 0);
    }

    #[pg_test]
    fn test_parse_decimal() {
        let q = TurboQuant::parse("3.14").unwrap();
        assert_eq!(q.value, 314);
        assert_eq!(q.scale, 2);
    }

    #[pg_test]
    fn test_display() {
        let q = TurboQuant::parse("3.14").unwrap();
        assert_eq!(q.to_display_string(), "3.14");
    }

    #[pg_test]
    fn test_add() {
        let a = TurboQuant::parse("1.50").unwrap();
        let b = TurboQuant::parse("2.25").unwrap();
        let result = turboquant_add(a, b);
        assert_eq!(result.to_display_string(), "3.75");
    }

    #[pg_test]
    fn test_mul() {
        let a = TurboQuant::parse("3.0").unwrap();
        let b = TurboQuant::parse("2.0").unwrap();
        let result = turboquant_mul(a, b);
        assert_eq!(result.to_display_string(), "6.00");
    }
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {}
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        vec![]
    }
}
