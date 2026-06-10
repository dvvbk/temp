//! Odpowiednik Rozszerzenia.Zaokragl - zaokrąglanie kwot jak w oryginale:
//! Decimal.Round(wartosc, miejsca, MidpointRounding.AwayFromZero).

use rust_decimal::{Decimal, RoundingStrategy};

pub fn zaokragl(wartosc: Decimal) -> Decimal {
    zaokragl_do(wartosc, 2)
}

pub fn zaokragl_do(wartosc: Decimal, miejsca: u32) -> Decimal {
    wartosc.round_dp_with_strategy(miejsca, RoundingStrategy::MidpointAwayFromZero)
}

#[cfg(test)]
mod testy {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn zaokragla_polowki_od_zera() {
        assert_eq!(zaokragl(dec!(1.005)), dec!(1.01));
        assert_eq!(zaokragl(dec!(-1.005)), dec!(-1.01));
        assert_eq!(zaokragl(dec!(2.675)), dec!(2.68));
        assert_eq!(zaokragl(dec!(1.004)), dec!(1.00));
    }
}
