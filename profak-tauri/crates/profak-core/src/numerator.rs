//! Numeracja dokumentów - odpowiednik DB.Numerator i DB.StanNumeratora.
//!
//! Format numeru zawiera wyrażenia w nawiasach kwadratowych, np. "FV/[Numer]/[Rok]".
//! Licznik jest prowadzony osobno dla każdej grupy (np. osobno dla każdego roku),
//! ponieważ stan numeratora jest indeksowany wygenerowanymi parametrami grupy.

use chrono::{Datelike, NaiveDate};
use regex::{Captures, Regex};
use rusqlite::{Connection, OptionalExtension};

use crate::model::PrzeznaczenieNumeratora;
use crate::{BladProFak, Wynik};

/// Wartości podstawiane do formatu numeru, pochodzące z faktury.
pub struct Podstawienia {
    pub data_wystawienia: NaiveDate,
    pub data_sprzedazy: NaiveDate,
}

impl Podstawienia {
    fn wartosc(&self, pole: &str) -> Option<String> {
        let pole = pole.to_lowercase();
        match pole.as_str() {
            "dzien" | "dzień" => Some(self.data_wystawienia.day().to_string()),
            "miesiac" | "miesiąc" => Some(self.data_wystawienia.month().to_string()),
            "rok" => Some(self.data_wystawienia.year().to_string()),
            "data" => Some(self.data_wystawienia.format("%Y-%m-%d").to_string()),
            "sprzedaz-dzien" | "sprzedaz-dzień" => Some(self.data_sprzedazy.day().to_string()),
            "sprzedaz-miesiac" | "sprzedaz-miesiąc" => Some(self.data_sprzedazy.month().to_string()),
            "sprzedaz-rok" => Some(self.data_sprzedazy.year().to_string()),
            "sprzedaz-data" => Some(self.data_sprzedazy.format("%Y-%m-%d").to_string()),
            _ => None,
        }
    }
}

fn wzorzec() -> Regex {
    Regex::new(r"\[(?P<nazwa>[\w-]+)(:(?P<format>[^\]]+))?\]").unwrap()
}

/// Formatowanie liczby według formatu w stylu .NET ("000" -> dopełnienie zerami).
fn formatuj_liczbe(liczba: i64, format: Option<&str>) -> String {
    match format {
        Some(f) if !f.trim().is_empty() => {
            let zera = f.chars().filter(|c| *c == '0').count();
            if zera > 0 {
                format!("{:0w$}", liczba, w = zera)
            } else {
                liczba.to_string()
            }
        }
        _ => liczba.to_string(),
    }
}

fn podstaw(format: &str, podstawienia: &Podstawienia, numer: Option<i64>) -> Wynik<String> {
    let rx = wzorzec();
    let mut blad: Option<String> = None;
    let wynik = rx.replace_all(format, |c: &Captures| {
        let nazwa = c.name("nazwa").map(|m| m.as_str()).unwrap_or("");
        let fmt = c.name("format").map(|m| m.as_str());
        if nazwa.eq_ignore_ascii_case("numer") {
            return match numer {
                Some(n) => formatuj_liczbe(n, fmt),
                None => String::new(),
            };
        }
        match podstawienia.wartosc(nazwa) {
            Some(w) => w,
            None => {
                blad = Some(format!("Nieznane wyrażenie numeratora \"{nazwa}\"."));
                String::new()
            }
        }
    });
    match blad {
        Some(b) => Err(BladProFak::Logika(b)),
        None => Ok(wynik.into_owned()),
    }
}

pub fn generuj_grupe(format: &str, grupa: Option<&str>, podstawienia: &Podstawienia) -> Wynik<String> {
    let wzor = match grupa {
        Some(g) if !g.is_empty() => g,
        _ => format,
    };
    podstaw(wzor, podstawienia, None)
}

pub fn generuj_numer(format: &str, podstawienia: &Podstawienia, licznik: i64) -> Wynik<String> {
    podstaw(format, podstawienia, Some(licznik))
}

/// Odpowiednik Numerator.NadajNumer: znajduje numerator dla przeznaczenia,
/// wyznacza grupę, zwiększa licznik i zwraca wygenerowany numer.
pub fn nadaj_numer(
    conn: &Connection,
    przeznaczenie: PrzeznaczenieNumeratora,
    podstawienia: &Podstawienia,
) -> Wynik<String> {
    let wiersz: Option<(i64, String, Option<String>)> = conn
        .query_row(
            "SELECT id, format, grupa FROM numerator WHERE przeznaczenie = ?1",
            [przeznaczenie.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let (numerator_id, format, grupa) = wiersz.ok_or_else(|| {
        BladProFak::Logika(format!(
            "Brak definicji numeratora \"{}\" - dodaj pozycję w spisie \"Numeracja\".",
            przeznaczenie.as_str()
        ))
    })?;

    let parametry = generuj_grupe(&format, grupa.as_deref(), podstawienia)?;

    let stan: Option<(i64, i64)> = conn
        .query_row(
            "SELECT id, ostatnia_wartosc FROM stan_numeratora WHERE numerator_id = ?1 AND parametry = ?2",
            rusqlite::params![numerator_id, parametry],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    let licznik = stan.map(|(_, w)| w).unwrap_or(0) + 1;
    let numer = generuj_numer(&format, podstawienia, licznik)?;

    match stan {
        Some((stan_id, _)) => {
            conn.execute(
                "UPDATE stan_numeratora SET ostatnia_wartosc = ?1 WHERE id = ?2",
                rusqlite::params![licznik, stan_id],
            )?;
        }
        None => {
            conn.execute(
                "INSERT INTO stan_numeratora (numerator_id, parametry, ostatnia_wartosc) VALUES (?1, ?2, ?3)",
                rusqlite::params![numerator_id, parametry, licznik],
            )?;
        }
    }

    Ok(numer)
}

#[cfg(test)]
mod testy {
    use super::*;

    fn podst(rok: i32, miesiac: u32) -> Podstawienia {
        let data = NaiveDate::from_ymd_opt(rok, miesiac, 15).unwrap();
        Podstawienia { data_wystawienia: data, data_sprzedazy: data }
    }

    #[test]
    fn generuje_numer_z_rokiem() {
        let n = generuj_numer("FV/[Numer]/[Rok]", &podst(2026, 6), 7).unwrap();
        assert_eq!(n, "FV/7/2026");
    }

    #[test]
    fn formatuje_numer_zerami() {
        let n = generuj_numer("FV/[Numer:000]/[Miesiąc]/[Rok]", &podst(2026, 6), 7).unwrap();
        assert_eq!(n, "FV/007/6/2026");
    }

    #[test]
    fn grupa_domyslnie_z_formatu_bez_numeru() {
        let g = generuj_grupe("FV/[Numer]/[Rok]", None, &podst(2026, 6)).unwrap();
        assert_eq!(g, "FV//2026");
    }

    #[test]
    fn nieznane_wyrazenie_zglasza_blad() {
        let w = generuj_numer("FV/[Numer]/[Kwartal]", &podst(2026, 6), 1);
        assert!(w.is_err());
    }

    #[test]
    fn licznik_rosnie_i_resetuje_sie_w_nowej_grupie() {
        let conn = crate::db::otworz_w_pamieci().unwrap();
        let p2026 = podst(2026, 6);
        let p2027 = podst(2027, 1);
        let n1 = nadaj_numer(&conn, PrzeznaczenieNumeratora::Faktura, &p2026).unwrap();
        let n2 = nadaj_numer(&conn, PrzeznaczenieNumeratora::Faktura, &p2026).unwrap();
        let n3 = nadaj_numer(&conn, PrzeznaczenieNumeratora::Faktura, &p2027).unwrap();
        assert_eq!(n1, "FV/1/2026");
        assert_eq!(n2, "FV/2/2026");
        assert_eq!(n3, "FV/1/2027");
    }
}
