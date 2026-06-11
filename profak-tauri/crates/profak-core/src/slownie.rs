//! Kwoty słownie po polsku - port klasy SlowniePL z ProFak.Wydruki.

use rust_decimal::Decimal;

const JEDNOSCI: [&str; 20] = [
    "zero", "jeden", "dwa", "trzy", "cztery", "pięć", "sześć", "siedem", "osiem", "dziewięć",
    "dziesięć", "jedenaście", "dwanaście", "trzynaście", "czternaście", "piętnaście",
    "szesnaście", "siedemnaście", "osiemnaście", "dziewiętnaście",
];
const DZIESIATKI: [&str; 10] = [
    "", "", "dwadzieścia", "trzydzieści", "czterdzieści", "pięćdziesiąt", "sześćdziesiąt",
    "siedemdziesiąt", "osiemdziesiąt", "dziewięćdziesiąt",
];
const SETKI: [&str; 10] = [
    "", "sto", "dwieście", "trzysta", "czterysta", "pięćset", "sześćset", "siedemset",
    "osiemset", "dziewięćset",
];
const TYSIACE: [[&str; 3]; 4] = [
    ["", "", ""],
    [" tysiąc", " tysiące", " tysięcy"],
    [" milion", " miliony", " milionów"],
    [" miliard", " miliardy", " miliardów"],
];

fn slownie_do_1000(mut wartosc: i64) -> String {
    if wartosc == 0 {
        return String::new();
    }
    let mut wynik = String::new();
    if wartosc >= 100 {
        wynik.push_str(SETKI[(wartosc / 100) as usize]);
        wartosc %= 100;
    }
    if wartosc >= 20 {
        if !wynik.is_empty() {
            wynik.push(' ');
        }
        wynik.push_str(DZIESIATKI[(wartosc / 10) as usize]);
        wartosc %= 10;
    }
    if wartosc > 0 {
        if !wynik.is_empty() {
            wynik.push(' ');
        }
        wynik.push_str(JEDNOSCI[wartosc as usize]);
    }
    wynik
}

fn slownie_rekurencyjnie(wynik: &mut String, mut wartosc: i64, rzad_wielkosci: usize) {
    if wartosc >= 1000 {
        slownie_rekurencyjnie(wynik, wartosc / 1000, rzad_wielkosci + 1);
        wartosc %= 1000;
    }
    if wartosc == 0 {
        return;
    }
    if !wynik.is_empty() {
        wynik.push(' ');
    }
    wynik.push_str(&slownie_do_1000(wartosc));

    let ostatnia_cyfra = wartosc % 10;
    let dwie_cyfry = wartosc % 100;
    // Odmiana: 0 = "tysiąc", 1 = "tysiące", 2 = "tysięcy" (logika jak w oryginale).
    let odmiana = if wartosc >= 100 {
        if dwie_cyfry >= 20 {
            if (2..=4).contains(&ostatnia_cyfra) { 1 } else { 2 }
        } else if (2..=4).contains(&dwie_cyfry) {
            1
        } else {
            2
        }
    } else if wartosc >= 20 {
        if (2..=4).contains(&ostatnia_cyfra) { 1 } else { 2 }
    } else if wartosc >= 5 {
        2
    } else if wartosc >= 2 {
        1
    } else {
        0
    };
    wynik.push_str(TYSIACE[rzad_wielkosci][odmiana]);
}

pub fn slownie(wartosc: i64) -> String {
    if wartosc == 0 {
        return JEDNOSCI[0].to_string();
    }
    let mut wynik = String::new();
    slownie_rekurencyjnie(&mut wynik, wartosc, 0);
    wynik
}

/// Kwota słownie z walutą, np. "sto dwadzieścia trzy zł i czterdzieści pięć gr".
pub fn slownie_kwota(kwota: Decimal, waluta: &str) -> String {
    let zlote = kwota.trunc();
    let grosze = ((kwota - zlote) * Decimal::ONE_HUNDRED).trunc();
    let zlote: i64 = zlote.try_into().unwrap_or(0);
    let grosze: i64 = grosze.try_into().unwrap_or(0);
    let mut wynik = format!("{} {}", slownie(zlote), waluta);
    if grosze > 0 {
        let koncowka = if waluta == "zł" {
            "gr".to_string()
        } else {
            format!("{waluta}/100")
        };
        wynik = format!("{} i {} {}", wynik, slownie(grosze), koncowka);
    }
    wynik
}

#[cfg(test)]
mod testy {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn liczby_proste() {
        assert_eq!(slownie(0), "zero");
        assert_eq!(slownie(7), "siedem");
        assert_eq!(slownie(15), "piętnaście");
        assert_eq!(slownie(21), "dwadzieścia jeden");
        assert_eq!(slownie(152), "sto pięćdziesiąt dwa");
    }

    #[test]
    fn odmiana_tysiecy() {
        assert_eq!(slownie(1000), "jeden tysiąc");
        assert_eq!(slownie(2000), "dwa tysiące");
        assert_eq!(slownie(5000), "pięć tysięcy");
        assert_eq!(slownie(12000), "dwanaście tysięcy");
        assert_eq!(slownie(22000), "dwadzieścia dwa tysiące");
        assert_eq!(slownie(1500), "jeden tysiąc pięćset");
        assert_eq!(slownie(2_000_000), "dwa miliony");
    }

    #[test]
    fn kwota_z_groszami() {
        assert_eq!(
            slownie_kwota(dec!(123.45), "zł"),
            "sto dwadzieścia trzy zł i czterdzieści pięć gr"
        );
        assert_eq!(slownie_kwota(dec!(246.00), "zł"), "dwieście czterdzieści sześć zł");
        assert_eq!(
            slownie_kwota(dec!(10.05), "EUR"),
            "dziesięć EUR i pięć EUR/100"
        );
    }
}
