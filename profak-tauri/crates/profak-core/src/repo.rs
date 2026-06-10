//! Operacje na bazie - CRUD słowników oraz operacje na fakturach
//! (zapis z przeliczeniem, wystawianie, korekta, kopia).

use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::faktura::{popraw_numeracje_pozycji, przelicz_ceny, przelicz_razem};
use crate::model::*;
use crate::numerator::{nadaj_numer, Podstawienia};
use crate::{BladProFak, Wynik};

fn dec(row: &Row, idx: usize) -> rusqlite::Result<Decimal> {
    let s: String = row.get(idx)?;
    Ok(Decimal::from_str(&s).unwrap_or(Decimal::ZERO))
}

fn dec_opt(row: &Row, idx: usize) -> rusqlite::Result<Option<Decimal>> {
    let s: Option<String> = row.get(idx)?;
    Ok(s.and_then(|s| Decimal::from_str(&s).ok()))
}

fn data(row: &Row, idx: usize) -> rusqlite::Result<NaiveDate> {
    let s: String = row.get(idx)?;
    Ok(NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap_or_default())
}

// ---------- Kontrahenci ----------

fn kontrahent_z_wiersza(row: &Row) -> rusqlite::Result<Kontrahent> {
    Ok(Kontrahent {
        id: row.get(0)?,
        nazwa: row.get(1)?,
        pelna_nazwa: row.get(2)?,
        nip: row.get(3)?,
        adres_rejestrowy: row.get(4)?,
        adres_korespondencyjny: row.get(5)?,
        rachunek_bankowy: row.get(6)?,
        nazwa_banku: row.get(7)?,
        telefon: row.get(8)?,
        email: row.get(9)?,
        uwagi_wewnetrzne: row.get(10)?,
        uwagi_publiczne: row.get(11)?,
        czy_archiwalny: row.get(12)?,
        czy_podmiot: row.get(13)?,
        czy_tp: row.get(14)?,
        sposob_platnosci_id: row.get(15)?,
        domyslna_waluta_id: row.get(16)?,
    })
}

const KONTRAHENT_KOLUMNY: &str = "id, nazwa, pelna_nazwa, nip, adres_rejestrowy, adres_korespondencyjny, \
    rachunek_bankowy, nazwa_banku, telefon, email, uwagi_wewnetrzne, uwagi_publiczne, \
    czy_archiwalny, czy_podmiot, czy_tp, sposob_platnosci_id, domyslna_waluta_id";

pub fn lista_kontrahentow(conn: &Connection) -> Wynik<Vec<Kontrahent>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {KONTRAHENT_KOLUMNY} FROM kontrahent ORDER BY nazwa COLLATE NOCASE"
    ))?;
    let wynik = stmt
        .query_map([], kontrahent_z_wiersza)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

pub fn pobierz_kontrahenta(conn: &Connection, id: i64) -> Wynik<Option<Kontrahent>> {
    let wynik = conn
        .query_row(
            &format!("SELECT {KONTRAHENT_KOLUMNY} FROM kontrahent WHERE id = ?1"),
            [id],
            kontrahent_z_wiersza,
        )
        .optional()?;
    Ok(wynik)
}

pub fn zapisz_kontrahenta(conn: &Connection, k: &Kontrahent) -> Wynik<i64> {
    if k.id == 0 {
        conn.execute(
            "INSERT INTO kontrahent (nazwa, pelna_nazwa, nip, adres_rejestrowy, adres_korespondencyjny, \
             rachunek_bankowy, nazwa_banku, telefon, email, uwagi_wewnetrzne, uwagi_publiczne, \
             czy_archiwalny, czy_podmiot, czy_tp, sposob_platnosci_id, domyslna_waluta_id) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            params![
                k.nazwa, k.pelna_nazwa, k.nip, k.adres_rejestrowy, k.adres_korespondencyjny,
                k.rachunek_bankowy, k.nazwa_banku, k.telefon, k.email, k.uwagi_wewnetrzne,
                k.uwagi_publiczne, k.czy_archiwalny, k.czy_podmiot, k.czy_tp,
                k.sposob_platnosci_id, k.domyslna_waluta_id
            ],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE kontrahent SET nazwa=?1, pelna_nazwa=?2, nip=?3, adres_rejestrowy=?4, \
             adres_korespondencyjny=?5, rachunek_bankowy=?6, nazwa_banku=?7, telefon=?8, email=?9, \
             uwagi_wewnetrzne=?10, uwagi_publiczne=?11, czy_archiwalny=?12, czy_podmiot=?13, \
             czy_tp=?14, sposob_platnosci_id=?15, domyslna_waluta_id=?16 WHERE id=?17",
            params![
                k.nazwa, k.pelna_nazwa, k.nip, k.adres_rejestrowy, k.adres_korespondencyjny,
                k.rachunek_bankowy, k.nazwa_banku, k.telefon, k.email, k.uwagi_wewnetrzne,
                k.uwagi_publiczne, k.czy_archiwalny, k.czy_podmiot, k.czy_tp,
                k.sposob_platnosci_id, k.domyslna_waluta_id, k.id
            ],
        )?;
        Ok(k.id)
    }
}

pub fn usun_kontrahenta(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("DELETE FROM kontrahent WHERE id = ?1", [id])?;
    Ok(())
}

// ---------- Towary ----------

fn towar_z_wiersza(row: &Row) -> rusqlite::Result<Towar> {
    let rodzaj: String = row.get(2)?;
    let sposob: String = row.get(5)?;
    Ok(Towar {
        id: row.get(0)?,
        nazwa: row.get(1)?,
        rodzaj: if rodzaj == "Usługa" { RodzajTowaru::Usluga } else { RodzajTowaru::Towar },
        cena_netto: dec(row, 3)?,
        cena_brutto: dec(row, 4)?,
        sposob_liczenia_ceny: match sposob.as_str() {
            "WedługBrutto" => SposobLiczeniaCenyTowaru::WedlugBrutto,
            "NarzutKwotowy" => SposobLiczeniaCenyTowaru::NarzutKwotowy,
            "NarzutProcentowy" => SposobLiczeniaCenyTowaru::NarzutProcentowy,
            _ => SposobLiczeniaCenyTowaru::WedlugNetto,
        },
        czy_archiwalny: row.get(6)?,
        gtu: row.get(7)?,
        stawka_ryczaltu: dec_opt(row, 8)?,
        stawka_vat_id: row.get(9)?,
        jednostka_miary_id: row.get(10)?,
    })
}

pub fn lista_towarow(conn: &Connection) -> Wynik<Vec<Towar>> {
    let mut stmt = conn.prepare(
        "SELECT id, nazwa, rodzaj, cena_netto, cena_brutto, sposob_liczenia_ceny, czy_archiwalny, \
         gtu, stawka_ryczaltu, stawka_vat_id, jednostka_miary_id FROM towar ORDER BY nazwa COLLATE NOCASE",
    )?;
    let wynik = stmt
        .query_map([], towar_z_wiersza)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

pub fn zapisz_towar(conn: &Connection, t: &Towar) -> Wynik<i64> {
    let rodzaj = match t.rodzaj {
        RodzajTowaru::Towar => "Towar",
        RodzajTowaru::Usluga => "Usługa",
    };
    let sposob = match t.sposob_liczenia_ceny {
        SposobLiczeniaCenyTowaru::WedlugNetto => "WedługNetto",
        SposobLiczeniaCenyTowaru::WedlugBrutto => "WedługBrutto",
        SposobLiczeniaCenyTowaru::NarzutKwotowy => "NarzutKwotowy",
        SposobLiczeniaCenyTowaru::NarzutProcentowy => "NarzutProcentowy",
    };
    if t.id == 0 {
        conn.execute(
            "INSERT INTO towar (nazwa, rodzaj, cena_netto, cena_brutto, sposob_liczenia_ceny, \
             czy_archiwalny, gtu, stawka_ryczaltu, stawka_vat_id, jednostka_miary_id) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                t.nazwa, rodzaj, t.cena_netto.to_string(), t.cena_brutto.to_string(), sposob,
                t.czy_archiwalny, t.gtu, t.stawka_ryczaltu.map(|d| d.to_string()),
                t.stawka_vat_id, t.jednostka_miary_id
            ],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE towar SET nazwa=?1, rodzaj=?2, cena_netto=?3, cena_brutto=?4, \
             sposob_liczenia_ceny=?5, czy_archiwalny=?6, gtu=?7, stawka_ryczaltu=?8, \
             stawka_vat_id=?9, jednostka_miary_id=?10 WHERE id=?11",
            params![
                t.nazwa, rodzaj, t.cena_netto.to_string(), t.cena_brutto.to_string(), sposob,
                t.czy_archiwalny, t.gtu, t.stawka_ryczaltu.map(|d| d.to_string()),
                t.stawka_vat_id, t.jednostka_miary_id, t.id
            ],
        )?;
        Ok(t.id)
    }
}

pub fn usun_towar(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("DELETE FROM towar WHERE id = ?1", [id])?;
    Ok(())
}

// ---------- Słowniki ----------

pub fn lista_stawek_vat(conn: &Connection) -> Wynik<Vec<StawkaVat>> {
    let mut stmt = conn.prepare("SELECT id, skrot, wartosc, czy_domyslna FROM stawka_vat ORDER BY id")?;
    let wynik = stmt
        .query_map([], |row| {
            Ok(StawkaVat {
                id: row.get(0)?,
                skrot: row.get(1)?,
                wartosc: dec(row, 2)?,
                czy_domyslna: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

pub fn zapisz_stawke_vat(conn: &Connection, s: &StawkaVat) -> Wynik<i64> {
    if s.id == 0 {
        conn.execute(
            "INSERT INTO stawka_vat (skrot, wartosc, czy_domyslna) VALUES (?1,?2,?3)",
            params![s.skrot, s.wartosc.to_string(), s.czy_domyslna],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE stawka_vat SET skrot=?1, wartosc=?2, czy_domyslna=?3 WHERE id=?4",
            params![s.skrot, s.wartosc.to_string(), s.czy_domyslna, s.id],
        )?;
        Ok(s.id)
    }
}

pub fn usun_stawke_vat(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("DELETE FROM stawka_vat WHERE id = ?1", [id])?;
    Ok(())
}

pub fn lista_jednostek(conn: &Connection) -> Wynik<Vec<JednostkaMiary>> {
    let mut stmt = conn.prepare(
        "SELECT id, skrot, nazwa, czy_domyslna, liczba_miejsc_po_przecinku FROM jednostka_miary ORDER BY id",
    )?;
    let wynik = stmt
        .query_map([], |row| {
            Ok(JednostkaMiary {
                id: row.get(0)?,
                skrot: row.get(1)?,
                nazwa: row.get(2)?,
                czy_domyslna: row.get(3)?,
                liczba_miejsc_po_przecinku: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

pub fn zapisz_jednostke(conn: &Connection, j: &JednostkaMiary) -> Wynik<i64> {
    if j.id == 0 {
        conn.execute(
            "INSERT INTO jednostka_miary (skrot, nazwa, czy_domyslna, liczba_miejsc_po_przecinku) VALUES (?1,?2,?3,?4)",
            params![j.skrot, j.nazwa, j.czy_domyslna, j.liczba_miejsc_po_przecinku],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE jednostka_miary SET skrot=?1, nazwa=?2, czy_domyslna=?3, liczba_miejsc_po_przecinku=?4 WHERE id=?5",
            params![j.skrot, j.nazwa, j.czy_domyslna, j.liczba_miejsc_po_przecinku, j.id],
        )?;
        Ok(j.id)
    }
}

pub fn usun_jednostke(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("DELETE FROM jednostka_miary WHERE id = ?1", [id])?;
    Ok(())
}

pub fn lista_walut(conn: &Connection) -> Wynik<Vec<Waluta>> {
    let mut stmt = conn.prepare("SELECT id, skrot, nazwa, czy_domyslna FROM waluta ORDER BY id")?;
    let wynik = stmt
        .query_map([], |row| {
            Ok(Waluta {
                id: row.get(0)?,
                skrot: row.get(1)?,
                nazwa: row.get(2)?,
                czy_domyslna: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

pub fn zapisz_walute(conn: &Connection, w: &Waluta) -> Wynik<i64> {
    if w.id == 0 {
        conn.execute(
            "INSERT INTO waluta (skrot, nazwa, czy_domyslna) VALUES (?1,?2,?3)",
            params![w.skrot, w.nazwa, w.czy_domyslna],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE waluta SET skrot=?1, nazwa=?2, czy_domyslna=?3 WHERE id=?4",
            params![w.skrot, w.nazwa, w.czy_domyslna, w.id],
        )?;
        Ok(w.id)
    }
}

pub fn usun_walute(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("DELETE FROM waluta WHERE id = ?1", [id])?;
    Ok(())
}

pub fn lista_sposobow_platnosci(conn: &Connection) -> Wynik<Vec<SposobPlatnosci>> {
    let mut stmt = conn.prepare(
        "SELECT id, nazwa, liczba_dni, czy_domyslny, czy_zaplacone FROM sposob_platnosci ORDER BY id",
    )?;
    let wynik = stmt
        .query_map([], |row| {
            Ok(SposobPlatnosci {
                id: row.get(0)?,
                nazwa: row.get(1)?,
                liczba_dni: row.get(2)?,
                czy_domyslny: row.get(3)?,
                czy_zaplacone: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

pub fn zapisz_sposob_platnosci(conn: &Connection, s: &SposobPlatnosci) -> Wynik<i64> {
    if s.id == 0 {
        conn.execute(
            "INSERT INTO sposob_platnosci (nazwa, liczba_dni, czy_domyslny, czy_zaplacone) VALUES (?1,?2,?3,?4)",
            params![s.nazwa, s.liczba_dni, s.czy_domyslny, s.czy_zaplacone],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE sposob_platnosci SET nazwa=?1, liczba_dni=?2, czy_domyslny=?3, czy_zaplacone=?4 WHERE id=?5",
            params![s.nazwa, s.liczba_dni, s.czy_domyslny, s.czy_zaplacone, s.id],
        )?;
        Ok(s.id)
    }
}

pub fn usun_sposob_platnosci(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("DELETE FROM sposob_platnosci WHERE id = ?1", [id])?;
    Ok(())
}

pub fn lista_numeratorow(conn: &Connection) -> Wynik<Vec<Numerator>> {
    let mut stmt = conn.prepare("SELECT id, przeznaczenie, format, grupa FROM numerator ORDER BY id")?;
    let wynik = stmt
        .query_map([], |row| {
            let przeznaczenie: String = row.get(1)?;
            Ok(Numerator {
                id: row.get(0)?,
                przeznaczenie: PrzeznaczenieNumeratora::from_str(&przeznaczenie),
                format: row.get(2)?,
                grupa: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

pub fn zapisz_numerator(conn: &Connection, n: &Numerator) -> Wynik<i64> {
    if n.id == 0 {
        conn.execute(
            "INSERT INTO numerator (przeznaczenie, format, grupa) VALUES (?1,?2,?3)",
            params![n.przeznaczenie.as_str(), n.format, n.grupa],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE numerator SET przeznaczenie=?1, format=?2, grupa=?3 WHERE id=?4",
            params![n.przeznaczenie.as_str(), n.format, n.grupa, n.id],
        )?;
        Ok(n.id)
    }
}

pub fn usun_numerator(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("DELETE FROM numerator WHERE id = ?1", [id])?;
    Ok(())
}

// ---------- Faktury ----------

const FAKTURA_KOLUMNY: &str = "id, numer, data_wystawienia, data_sprzedazy, data_wprowadzenia, \
    termin_platnosci, nip_sprzedawcy, nazwa_sprzedawcy, dane_sprzedawcy, nip_nabywcy, nazwa_nabywcy, \
    dane_nabywcy, rachunek_bankowy, nazwa_banku, uwagi_publiczne, uwagi_wewnetrzne, razem_netto, \
    razem_vat, razem_brutto, kurs_waluty, opis_sposobu_platnosci, rodzaj, czy_wartosci_reczne, \
    procedura_marzy, numer_ksef, sprzedawca_id, nabywca_id, faktura_korygowana_id, \
    faktura_korygujaca_id, waluta_id, sposob_platnosci_id";

fn faktura_z_wiersza(row: &Row) -> rusqlite::Result<Faktura> {
    let rodzaj: String = row.get(21)?;
    let procedura: String = row.get(23)?;
    Ok(Faktura {
        id: row.get(0)?,
        numer: row.get(1)?,
        data_wystawienia: data(row, 2)?,
        data_sprzedazy: data(row, 3)?,
        data_wprowadzenia: data(row, 4)?,
        termin_platnosci: data(row, 5)?,
        nip_sprzedawcy: row.get(6)?,
        nazwa_sprzedawcy: row.get(7)?,
        dane_sprzedawcy: row.get(8)?,
        nip_nabywcy: row.get(9)?,
        nazwa_nabywcy: row.get(10)?,
        dane_nabywcy: row.get(11)?,
        rachunek_bankowy: row.get(12)?,
        nazwa_banku: row.get(13)?,
        uwagi_publiczne: row.get(14)?,
        uwagi_wewnetrzne: row.get(15)?,
        razem_netto: dec(row, 16)?,
        razem_vat: dec(row, 17)?,
        razem_brutto: dec(row, 18)?,
        kurs_waluty: dec(row, 19)?,
        opis_sposobu_platnosci: row.get(20)?,
        rodzaj: RodzajFaktury::from_str(&rodzaj),
        czy_wartosci_reczne: row.get(22)?,
        procedura_marzy: ProceduraMarzy::from_str(&procedura),
        numer_ksef: row.get(24)?,
        sprzedawca_id: row.get(25)?,
        nabywca_id: row.get(26)?,
        faktura_korygowana_id: row.get(27)?,
        faktura_korygujaca_id: row.get(28)?,
        waluta_id: row.get(29)?,
        sposob_platnosci_id: row.get(30)?,
    })
}

/// Lista faktur, opcjonalnie zawężona do sprzedaży lub zakupu.
pub fn lista_faktur(conn: &Connection, czy_sprzedaz: Option<bool>) -> Wynik<Vec<FakturaListaWiersz>> {
    let rodzaje_sprzedazy = "'Sprzedaż','KorektaSprzedaży','Proforma','VatMarża','KorektaVatMarży','Rachunek','KorektaRachunku'";
    let filtr = match czy_sprzedaz {
        Some(true) => format!("WHERE f.rodzaj IN ({rodzaje_sprzedazy})"),
        Some(false) => format!("WHERE f.rodzaj NOT IN ({rodzaje_sprzedazy})"),
        None => String::new(),
    };
    let sql = format!(
        "SELECT {kolumny}, COALESCE(w.skrot, '') AS waluta_skrot \
         FROM faktura f LEFT JOIN waluta w ON w.id = f.waluta_id {filtr} \
         ORDER BY f.data_wystawienia DESC, f.id DESC",
        kolumny = FAKTURA_KOLUMNY
            .split(", ")
            .map(|k| format!("f.{k}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut wynik = stmt
        .query_map([], |row| {
            let faktura = faktura_z_wiersza(row)?;
            let waluta_skrot: String = row.get(31)?;
            Ok(FakturaListaWiersz {
                faktura,
                suma_wplat: Decimal::ZERO,
                pozostalo_do_zaplaty: Decimal::ZERO,
                waluta_skrot,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Sumy wpłat - kwoty trzymamy jako TEXT, więc sumowanie wykonujemy w Rust.
    let mut stmt = conn.prepare("SELECT faktura_id, kwota FROM wplata")?;
    let wplaty = stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, dec(row, 1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for wiersz in wynik.iter_mut() {
        let suma: Decimal = wplaty
            .iter()
            .filter(|(id, _)| *id == wiersz.faktura.id)
            .map(|(_, kwota)| *kwota)
            .sum();
        wiersz.suma_wplat = suma;
        wiersz.pozostalo_do_zaplaty = (wiersz.faktura.razem_brutto - suma).max(Decimal::ZERO);
    }
    Ok(wynik)
}

pub fn pobierz_fakture(conn: &Connection, id: i64) -> Wynik<Option<Faktura>> {
    let wynik = conn
        .query_row(
            &format!("SELECT {FAKTURA_KOLUMNY} FROM faktura WHERE id = ?1"),
            [id],
            faktura_z_wiersza,
        )
        .optional()?;
    Ok(wynik)
}

fn pozycja_z_wiersza(row: &Row) -> rusqlite::Result<PozycjaFaktury> {
    Ok(PozycjaFaktury {
        id: row.get(0)?,
        faktura_id: row.get(1)?,
        towar_id: row.get(2)?,
        opis: row.get(3)?,
        cena_netto: dec(row, 4)?,
        cena_vat: dec(row, 5)?,
        cena_brutto: dec(row, 6)?,
        ilosc: dec(row, 7)?,
        wartosc_netto: dec(row, 8)?,
        wartosc_vat: dec(row, 9)?,
        wartosc_brutto: dec(row, 10)?,
        czy_wedlug_cen_brutto: row.get(11)?,
        czy_wartosci_reczne: row.get(12)?,
        stawka_vat_id: row.get(13)?,
        jednostka_miary_id: row.get(14)?,
        lp: row.get(15)?,
        czy_przed_korekta: row.get(16)?,
        gtu: row.get(17)?,
        stawka_ryczaltu: dec_opt(row, 18)?,
        rabat_procent: dec(row, 19)?,
        rabat_cena: dec(row, 20)?,
        rabat_wartosc: dec(row, 21)?,
        cena_zakupu_dla_marzy: dec(row, 22)?,
    })
}

pub fn pozycje_faktury(conn: &Connection, faktura_id: i64) -> Wynik<Vec<PozycjaFaktury>> {
    let mut stmt = conn.prepare(
        "SELECT id, faktura_id, towar_id, opis, cena_netto, cena_vat, cena_brutto, ilosc, \
         wartosc_netto, wartosc_vat, wartosc_brutto, czy_wedlug_cen_brutto, czy_wartosci_reczne, \
         stawka_vat_id, jednostka_miary_id, lp, czy_przed_korekta, gtu, stawka_ryczaltu, \
         rabat_procent, rabat_cena, rabat_wartosc, cena_zakupu_dla_marzy \
         FROM pozycja_faktury WHERE faktura_id = ?1 ORDER BY czy_przed_korekta DESC, lp, id",
    )?;
    let wynik = stmt
        .query_map([faktura_id], pozycja_z_wiersza)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

pub fn wplaty_faktury(conn: &Connection, faktura_id: i64) -> Wynik<Vec<Wplata>> {
    let mut stmt = conn.prepare(
        "SELECT id, faktura_id, data, kwota, uwagi, czy_rozliczenie FROM wplata WHERE faktura_id = ?1 ORDER BY data, id",
    )?;
    let wynik = stmt
        .query_map([faktura_id], |row| {
            Ok(Wplata {
                id: row.get(0)?,
                faktura_id: row.get(1)?,
                data: data(row, 2)?,
                kwota: dec(row, 3)?,
                uwagi: row.get(4)?,
                czy_rozliczenie: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(wynik)
}

fn stawka_vat_procent(conn: &Connection, stawka_vat_id: Option<i64>) -> Wynik<Decimal> {
    match stawka_vat_id {
        None => Ok(Decimal::ZERO),
        Some(id) => {
            let wartosc: Option<String> = conn
                .query_row("SELECT wartosc FROM stawka_vat WHERE id = ?1", [id], |r| r.get(0))
                .optional()?;
            Ok(wartosc
                .and_then(|w| Decimal::from_str(&w).ok())
                .unwrap_or(Decimal::ZERO))
        }
    }
}

fn wstaw_lub_zaktualizuj_pozycje(conn: &Connection, p: &PozycjaFaktury) -> Wynik<i64> {
    if p.id == 0 {
        conn.execute(
            "INSERT INTO pozycja_faktury (faktura_id, towar_id, opis, cena_netto, cena_vat, cena_brutto, \
             ilosc, wartosc_netto, wartosc_vat, wartosc_brutto, czy_wedlug_cen_brutto, czy_wartosci_reczne, \
             stawka_vat_id, jednostka_miary_id, lp, czy_przed_korekta, gtu, stawka_ryczaltu, rabat_procent, \
             rabat_cena, rabat_wartosc, cena_zakupu_dla_marzy) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22)",
            params![
                p.faktura_id, p.towar_id, p.opis, p.cena_netto.to_string(), p.cena_vat.to_string(),
                p.cena_brutto.to_string(), p.ilosc.to_string(), p.wartosc_netto.to_string(),
                p.wartosc_vat.to_string(), p.wartosc_brutto.to_string(), p.czy_wedlug_cen_brutto,
                p.czy_wartosci_reczne, p.stawka_vat_id, p.jednostka_miary_id, p.lp, p.czy_przed_korekta,
                p.gtu, p.stawka_ryczaltu.map(|d| d.to_string()), p.rabat_procent.to_string(),
                p.rabat_cena.to_string(), p.rabat_wartosc.to_string(), p.cena_zakupu_dla_marzy.to_string()
            ],
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE pozycja_faktury SET faktura_id=?1, towar_id=?2, opis=?3, cena_netto=?4, cena_vat=?5, \
             cena_brutto=?6, ilosc=?7, wartosc_netto=?8, wartosc_vat=?9, wartosc_brutto=?10, \
             czy_wedlug_cen_brutto=?11, czy_wartosci_reczne=?12, stawka_vat_id=?13, jednostka_miary_id=?14, \
             lp=?15, czy_przed_korekta=?16, gtu=?17, stawka_ryczaltu=?18, rabat_procent=?19, rabat_cena=?20, \
             rabat_wartosc=?21, cena_zakupu_dla_marzy=?22 WHERE id=?23",
            params![
                p.faktura_id, p.towar_id, p.opis, p.cena_netto.to_string(), p.cena_vat.to_string(),
                p.cena_brutto.to_string(), p.ilosc.to_string(), p.wartosc_netto.to_string(),
                p.wartosc_vat.to_string(), p.wartosc_brutto.to_string(), p.czy_wedlug_cen_brutto,
                p.czy_wartosci_reczne, p.stawka_vat_id, p.jednostka_miary_id, p.lp, p.czy_przed_korekta,
                p.gtu, p.stawka_ryczaltu.map(|d| d.to_string()), p.rabat_procent.to_string(),
                p.rabat_cena.to_string(), p.rabat_wartosc.to_string(), p.cena_zakupu_dla_marzy.to_string(),
                p.id
            ],
        )?;
        Ok(p.id)
    }
}

fn wstaw_lub_zaktualizuj_fakture(conn: &Connection, f: &Faktura) -> Wynik<i64> {
    let parametry = params![
        f.numer,
        f.data_wystawienia.format("%Y-%m-%d").to_string(),
        f.data_sprzedazy.format("%Y-%m-%d").to_string(),
        f.data_wprowadzenia.format("%Y-%m-%d").to_string(),
        f.termin_platnosci.format("%Y-%m-%d").to_string(),
        f.nip_sprzedawcy, f.nazwa_sprzedawcy, f.dane_sprzedawcy,
        f.nip_nabywcy, f.nazwa_nabywcy, f.dane_nabywcy,
        f.rachunek_bankowy, f.nazwa_banku, f.uwagi_publiczne, f.uwagi_wewnetrzne,
        f.razem_netto.to_string(), f.razem_vat.to_string(), f.razem_brutto.to_string(),
        f.kurs_waluty.to_string(), f.opis_sposobu_platnosci, f.rodzaj.as_str(),
        f.czy_wartosci_reczne, f.procedura_marzy.as_str(), f.numer_ksef,
        f.sprzedawca_id, f.nabywca_id, f.faktura_korygowana_id, f.faktura_korygujaca_id,
        f.waluta_id, f.sposob_platnosci_id
    ];
    if f.id == 0 {
        conn.execute(
            "INSERT INTO faktura (numer, data_wystawienia, data_sprzedazy, data_wprowadzenia, \
             termin_platnosci, nip_sprzedawcy, nazwa_sprzedawcy, dane_sprzedawcy, nip_nabywcy, \
             nazwa_nabywcy, dane_nabywcy, rachunek_bankowy, nazwa_banku, uwagi_publiczne, \
             uwagi_wewnetrzne, razem_netto, razem_vat, razem_brutto, kurs_waluty, \
             opis_sposobu_platnosci, rodzaj, czy_wartosci_reczne, procedura_marzy, numer_ksef, \
             sprzedawca_id, nabywca_id, faktura_korygowana_id, faktura_korygujaca_id, waluta_id, \
             sposob_platnosci_id) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30)",
            parametry,
        )?;
        Ok(conn.last_insert_rowid())
    } else {
        conn.execute(
            "UPDATE faktura SET numer=?1, data_wystawienia=?2, data_sprzedazy=?3, data_wprowadzenia=?4, \
             termin_platnosci=?5, nip_sprzedawcy=?6, nazwa_sprzedawcy=?7, dane_sprzedawcy=?8, \
             nip_nabywcy=?9, nazwa_nabywcy=?10, dane_nabywcy=?11, rachunek_bankowy=?12, nazwa_banku=?13, \
             uwagi_publiczne=?14, uwagi_wewnetrzne=?15, razem_netto=?16, razem_vat=?17, razem_brutto=?18, \
             kurs_waluty=?19, opis_sposobu_platnosci=?20, rodzaj=?21, czy_wartosci_reczne=?22, \
             procedura_marzy=?23, numer_ksef=?24, sprzedawca_id=?25, nabywca_id=?26, \
             faktura_korygowana_id=?27, faktura_korygujaca_id=?28, waluta_id=?29, sposob_platnosci_id=?30 \
             WHERE id=?31",
            params![
                f.numer,
                f.data_wystawienia.format("%Y-%m-%d").to_string(),
                f.data_sprzedazy.format("%Y-%m-%d").to_string(),
                f.data_wprowadzenia.format("%Y-%m-%d").to_string(),
                f.termin_platnosci.format("%Y-%m-%d").to_string(),
                f.nip_sprzedawcy, f.nazwa_sprzedawcy, f.dane_sprzedawcy,
                f.nip_nabywcy, f.nazwa_nabywcy, f.dane_nabywcy,
                f.rachunek_bankowy, f.nazwa_banku, f.uwagi_publiczne, f.uwagi_wewnetrzne,
                f.razem_netto.to_string(), f.razem_vat.to_string(), f.razem_brutto.to_string(),
                f.kurs_waluty.to_string(), f.opis_sposobu_platnosci, f.rodzaj.as_str(),
                f.czy_wartosci_reczne, f.procedura_marzy.as_str(), f.numer_ksef,
                f.sprzedawca_id, f.nabywca_id, f.faktura_korygowana_id, f.faktura_korygujaca_id,
                f.waluta_id, f.sposob_platnosci_id, f.id
            ],
        )?;
        Ok(f.id)
    }
}

/// Przelicza fakturę i pozycje bez zapisu - do podglądu na żywo w edytorze.
pub fn przelicz_fakture(
    conn: &Connection,
    mut faktura: Faktura,
    mut pozycje: Vec<PozycjaFaktury>,
) -> Wynik<(Faktura, Vec<PozycjaFaktury>)> {
    let vat_marza = matches!(
        faktura.rodzaj,
        RodzajFaktury::VatMarza | RodzajFaktury::KorektaVatMarzy
    );
    for pozycja in pozycje.iter_mut() {
        let procent = stawka_vat_procent(conn, pozycja.stawka_vat_id)?;
        przelicz_ceny(pozycja, procent, vat_marza);
    }
    popraw_numeracje_pozycji(faktura.rodzaj, &mut pozycje);
    przelicz_razem(&mut faktura, &pozycje);
    Ok((faktura, pozycje))
}

/// Zapisuje fakturę wraz z pozycjami (z pełnym przeliczeniem).
/// Pozycje nieobecne na liście są usuwane z bazy.
pub fn zapisz_fakture(
    conn: &mut Connection,
    faktura: Faktura,
    pozycje: Vec<PozycjaFaktury>,
) -> Wynik<i64> {
    let (mut faktura, mut pozycje) = przelicz_fakture(conn, faktura, pozycje)?;
    let tx = conn.transaction()?;
    {
        let id = wstaw_lub_zaktualizuj_fakture(&tx, &faktura)?;
        faktura.id = id;
        let zachowane: Vec<i64> = pozycje.iter().map(|p| p.id).filter(|i| *i != 0).collect();
        let istniejace: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT id FROM pozycja_faktury WHERE faktura_id = ?1")?;
            let wiersze = stmt
                .query_map([id], |r| r.get(0))?
                .collect::<rusqlite::Result<Vec<i64>>>()?;
            wiersze
        };
        for stare_id in istniejace {
            if !zachowane.contains(&stare_id) {
                tx.execute("DELETE FROM pozycja_faktury WHERE id = ?1", [stare_id])?;
            }
        }
        for pozycja in pozycje.iter_mut() {
            pozycja.faktura_id = id;
            pozycja.id = wstaw_lub_zaktualizuj_pozycje(&tx, pozycja)?;
        }
    }
    tx.commit()?;
    Ok(faktura.id)
}

/// Odpowiednik Faktura.ZakonczWystawianie: nadaje numer z numeratora
/// (jeśli jeszcze nie nadano) i dodaje wpłatę przy płatności gotówką/kartą.
pub fn wystaw_fakture(conn: &mut Connection, faktura_id: i64) -> Wynik<Faktura> {
    let mut faktura = pobierz_fakture(conn, faktura_id)?
        .ok_or_else(|| BladProFak::Logika("Nie znaleziono faktury.".into()))?;

    let tx = conn.transaction()?;
    {
        if faktura.numer.is_empty() {
            if let Some(przeznaczenie) = faktura.rodzaj.numerator() {
                let podstawienia = Podstawienia {
                    data_wystawienia: faktura.data_wystawienia,
                    data_sprzedazy: faktura.data_sprzedazy,
                };
                faktura.numer = nadaj_numer(&tx, przeznaczenie, &podstawienia)?;
                tx.execute(
                    "UPDATE faktura SET numer = ?1 WHERE id = ?2",
                    params![faktura.numer, faktura.id],
                )?;
            }
        }

        // DodajWplateJesliTrzeba
        if let Some(sp_id) = faktura.sposob_platnosci_id {
            let czy_zaplacone: bool = tx
                .query_row(
                    "SELECT czy_zaplacone FROM sposob_platnosci WHERE id = ?1",
                    [sp_id],
                    |r| r.get(0),
                )
                .optional()?
                .unwrap_or(false);
            if czy_zaplacone {
                let wplaty: Vec<Decimal> = {
                    let mut stmt = tx.prepare("SELECT kwota FROM wplata WHERE faktura_id = ?1")?;
                    let wiersze = stmt
                        .query_map([faktura.id], |r| dec(r, 0))?
                        .collect::<rusqlite::Result<Vec<_>>>()?;
                    wiersze
                };
                let suma: Decimal = wplaty.into_iter().sum();
                if suma <= Decimal::ZERO {
                    tx.execute(
                        "INSERT INTO wplata (faktura_id, data, kwota, uwagi, czy_rozliczenie) VALUES (?1, ?2, ?3, '', 0)",
                        params![
                            faktura.id,
                            faktura.data_wystawienia.format("%Y-%m-%d").to_string(),
                            faktura.razem_brutto.to_string()
                        ],
                    )?;
                }
            }
        }
    }
    tx.commit()?;
    Ok(faktura)
}

pub fn usun_fakture(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("UPDATE faktura SET faktura_korygowana_id = NULL WHERE faktura_korygowana_id = ?1", [id])?;
    conn.execute("UPDATE faktura SET faktura_korygujaca_id = NULL WHERE faktura_korygujaca_id = ?1", [id])?;
    conn.execute("DELETE FROM faktura WHERE id = ?1", [id])?;
    Ok(())
}

pub fn dodaj_wplate(conn: &Connection, wplata: &Wplata) -> Wynik<i64> {
    conn.execute(
        "INSERT INTO wplata (faktura_id, data, kwota, uwagi, czy_rozliczenie) VALUES (?1,?2,?3,?4,?5)",
        params![
            wplata.faktura_id,
            wplata.data.format("%Y-%m-%d").to_string(),
            wplata.kwota.to_string(),
            wplata.uwagi,
            wplata.czy_rozliczenie
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn usun_wplate(conn: &Connection, id: i64) -> Wynik<()> {
    conn.execute("DELETE FROM wplata WHERE id = ?1", [id])?;
    Ok(())
}

/// Odpowiednik Faktura.PrzygotujKorekte: tworzy fakturę korygującą z pozycjami
/// "przed korektą" (ujemnymi) i "po korekcie".
pub fn przygotuj_korekte(conn: &mut Connection, faktura_id: i64) -> Wynik<i64> {
    let mut bazowa = pobierz_fakture(conn, faktura_id)?
        .ok_or_else(|| BladProFak::Logika("Nie znaleziono faktury.".into()))?;

    // Korekta zawsze od ostatniej faktury w łańcuchu korekt.
    while let Some(korygujaca_id) = bazowa.faktura_korygujaca_id {
        bazowa = pobierz_fakture(conn, korygujaca_id)?
            .ok_or_else(|| BladProFak::Logika("Nie znaleziono faktury korygującej.".into()))?;
    }

    let rodzaj_korekty = match bazowa.rodzaj {
        RodzajFaktury::Sprzedaz | RodzajFaktury::KorektaSprzedazy => RodzajFaktury::KorektaSprzedazy,
        RodzajFaktury::Rachunek => RodzajFaktury::KorektaRachunku,
        RodzajFaktury::Zakup | RodzajFaktury::KorektaZakupu => RodzajFaktury::KorektaZakupu,
        RodzajFaktury::VatMarza | RodzajFaktury::KorektaVatMarzy => RodzajFaktury::KorektaVatMarzy,
        rodzaj => {
            return Err(BladProFak::Logika(format!(
                "Nie można korygować faktury {}.",
                rodzaj.as_str()
            )))
        }
    };

    let mut korekta = Faktura {
        rodzaj: rodzaj_korekty,
        data_sprzedazy: bazowa.data_sprzedazy,
        faktura_korygowana_id: Some(bazowa.id),
        nip_sprzedawcy: bazowa.nip_sprzedawcy.clone(),
        nazwa_sprzedawcy: bazowa.nazwa_sprzedawcy.clone(),
        dane_sprzedawcy: bazowa.dane_sprzedawcy.clone(),
        nip_nabywcy: bazowa.nip_nabywcy.clone(),
        nazwa_nabywcy: bazowa.nazwa_nabywcy.clone(),
        dane_nabywcy: bazowa.dane_nabywcy.clone(),
        rachunek_bankowy: bazowa.rachunek_bankowy.clone(),
        uwagi_publiczne: bazowa.uwagi_publiczne.clone(),
        kurs_waluty: bazowa.kurs_waluty,
        opis_sposobu_platnosci: bazowa.opis_sposobu_platnosci.clone(),
        sprzedawca_id: bazowa.sprzedawca_id,
        nabywca_id: bazowa.nabywca_id,
        waluta_id: bazowa.waluta_id,
        sposob_platnosci_id: bazowa.sposob_platnosci_id,
        czy_wartosci_reczne: bazowa.czy_wartosci_reczne,
        procedura_marzy: bazowa.procedura_marzy,
        ..Faktura::default()
    };

    let stare_pozycje = pozycje_faktury(conn, bazowa.id)?;

    let tx = conn.transaction()?;
    {
        let korekta_id = wstaw_lub_zaktualizuj_fakture(&tx, &korekta)?;
        korekta.id = korekta_id;

        for stara in stare_pozycje.iter().filter(|p| !p.czy_przed_korekta) {
            let mut przed = stara.clone();
            przed.id = 0;
            przed.faktura_id = korekta_id;
            przed.ilosc = -stara.ilosc;
            przed.wartosc_netto = -stara.wartosc_netto;
            przed.wartosc_vat = -stara.wartosc_vat;
            przed.wartosc_brutto = -stara.wartosc_brutto;
            przed.czy_przed_korekta = true;
            wstaw_lub_zaktualizuj_pozycje(&tx, &przed)?;

            let mut po = stara.clone();
            po.id = 0;
            po.faktura_id = korekta_id;
            po.czy_przed_korekta = false;
            wstaw_lub_zaktualizuj_pozycje(&tx, &po)?;
        }

        tx.execute(
            "UPDATE faktura SET faktura_korygujaca_id = ?1 WHERE id = ?2",
            params![korekta_id, bazowa.id],
        )?;
    }
    tx.commit()?;
    Ok(korekta.id)
}

/// Odpowiednik Faktura.PrzygotujPodobna: nowa faktura z tymi samymi danymi i pozycjami.
pub fn przygotuj_podobna(conn: &mut Connection, faktura_id: i64) -> Wynik<i64> {
    let wzor = pobierz_fakture(conn, faktura_id)?
        .ok_or_else(|| BladProFak::Logika("Nie znaleziono faktury.".into()))?;

    let mut nowa = Faktura {
        rodzaj: wzor.rodzaj,
        rachunek_bankowy: wzor.rachunek_bankowy.clone(),
        uwagi_publiczne: wzor.uwagi_publiczne.clone(),
        uwagi_wewnetrzne: wzor.uwagi_wewnetrzne.clone(),
        kurs_waluty: wzor.kurs_waluty,
        opis_sposobu_platnosci: wzor.opis_sposobu_platnosci.clone(),
        waluta_id: wzor.waluta_id,
        sposob_platnosci_id: wzor.sposob_platnosci_id,
        czy_wartosci_reczne: wzor.czy_wartosci_reczne,
        procedura_marzy: wzor.procedura_marzy,
        ..Faktura::default()
    };
    if wzor.rodzaj.numerator().is_none() {
        nowa.numer = wzor.numer.clone();
    }
    if wzor.rodzaj.czy_zakup() {
        nowa.nip_sprzedawcy = wzor.nip_sprzedawcy.clone();
        nowa.nazwa_sprzedawcy = wzor.nazwa_sprzedawcy.clone();
        nowa.dane_sprzedawcy = wzor.dane_sprzedawcy.clone();
        nowa.sprzedawca_id = wzor.sprzedawca_id;
    } else {
        nowa.nip_nabywcy = wzor.nip_nabywcy.clone();
        nowa.nazwa_nabywcy = wzor.nazwa_nabywcy.clone();
        nowa.dane_nabywcy = wzor.dane_nabywcy.clone();
        nowa.nabywca_id = wzor.nabywca_id;
        nowa.nip_sprzedawcy = wzor.nip_sprzedawcy.clone();
        nowa.nazwa_sprzedawcy = wzor.nazwa_sprzedawcy.clone();
        nowa.dane_sprzedawcy = wzor.dane_sprzedawcy.clone();
        nowa.sprzedawca_id = wzor.sprzedawca_id;
    }

    if let Some(sp_id) = wzor.sposob_platnosci_id {
        let liczba_dni: Option<i64> = conn
            .query_row("SELECT liczba_dni FROM sposob_platnosci WHERE id = ?1", [sp_id], |r| r.get(0))
            .optional()?;
        if let Some(dni) = liczba_dni {
            nowa.termin_platnosci = nowa.data_wystawienia + chrono::Duration::days(dni);
        }
    }

    let stare_pozycje = pozycje_faktury(conn, wzor.id)?;
    let nowe_pozycje: Vec<PozycjaFaktury> = stare_pozycje
        .into_iter()
        .filter(|p| !p.czy_przed_korekta)
        .enumerate()
        .map(|(i, mut p)| {
            p.id = 0;
            p.lp = (i + 1) as i64;
            p
        })
        .collect();

    zapisz_fakture(conn, nowa, nowe_pozycje)
}

#[cfg(test)]
mod testy {
    use super::*;
    use rust_decimal_macros::dec as d;

    fn baza() -> Connection {
        crate::db::otworz_w_pamieci().unwrap()
    }

    fn stawka_23(conn: &Connection) -> i64 {
        conn.query_row("SELECT id FROM stawka_vat WHERE skrot = '23%'", [], |r| r.get(0))
            .unwrap()
    }

    fn prosta_faktura(conn: &mut Connection) -> i64 {
        let faktura = Faktura::default();
        let pozycja = PozycjaFaktury {
            opis: "Usługa programistyczna".into(),
            cena_netto: d!(100),
            ilosc: d!(2),
            stawka_vat_id: Some(stawka_23(conn)),
            ..PozycjaFaktury::default()
        };
        zapisz_fakture(conn, faktura, vec![pozycja]).unwrap()
    }

    #[test]
    fn dane_startowe_sa_ladowane() {
        let conn = baza();
        let stawki = lista_stawek_vat(&conn).unwrap();
        assert_eq!(stawki.len(), 6);
        let numeratory = lista_numeratorow(&conn).unwrap();
        assert_eq!(numeratory.len(), 8);
        let sposoby = lista_sposobow_platnosci(&conn).unwrap();
        assert_eq!(sposoby.len(), 6);
    }

    #[test]
    fn kontrahent_crud() {
        let conn = baza();
        let mut k = Kontrahent {
            nazwa: "ACME".into(),
            nip: "1234567890".into(),
            ..Kontrahent::default()
        };
        k.id = zapisz_kontrahenta(&conn, &k).unwrap();
        assert!(k.id > 0);
        k.nazwa = "ACME Sp. z o.o.".into();
        zapisz_kontrahenta(&conn, &k).unwrap();
        let lista = lista_kontrahentow(&conn).unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].nazwa, "ACME Sp. z o.o.");
        usun_kontrahenta(&conn, k.id).unwrap();
        assert!(lista_kontrahentow(&conn).unwrap().is_empty());
    }

    #[test]
    fn zapis_faktury_przelicza_pozycje_i_sumy() {
        let mut conn = baza();
        let id = prosta_faktura(&mut conn);
        let faktura = pobierz_fakture(&conn, id).unwrap().unwrap();
        assert_eq!(faktura.razem_netto, d!(200.00));
        assert_eq!(faktura.razem_vat, d!(46.00));
        assert_eq!(faktura.razem_brutto, d!(246.00));
        let pozycje = pozycje_faktury(&conn, id).unwrap();
        assert_eq!(pozycje.len(), 1);
        assert_eq!(pozycje[0].lp, 1);
        assert_eq!(pozycje[0].cena_brutto, d!(123.00));
    }

    #[test]
    fn wystawienie_nadaje_numer_i_dodaje_wplate_gotowkowa() {
        let mut conn = baza();
        let gotowka: i64 = conn
            .query_row("SELECT id FROM sposob_platnosci WHERE nazwa = 'Gotówka'", [], |r| r.get(0))
            .unwrap();
        let faktura = Faktura { sposob_platnosci_id: Some(gotowka), ..Faktura::default() };
        let pozycja = PozycjaFaktury {
            opis: "Towar".into(),
            cena_netto: d!(100),
            ilosc: d!(1),
            stawka_vat_id: Some(stawka_23(&conn)),
            ..PozycjaFaktury::default()
        };
        let id = zapisz_fakture(&mut conn, faktura, vec![pozycja]).unwrap();
        let wystawiona = wystaw_fakture(&mut conn, id).unwrap();
        let rok = chrono::Local::now().format("%Y").to_string();
        assert_eq!(wystawiona.numer, format!("FV/1/{rok}"));
        let wplaty = wplaty_faktury(&conn, id).unwrap();
        assert_eq!(wplaty.len(), 1);
        assert_eq!(wplaty[0].kwota, d!(123.00));
        // Ponowne wystawienie nie zmienia numeru ani nie dubluje wpłaty.
        let ponownie = wystaw_fakture(&mut conn, id).unwrap();
        assert_eq!(ponownie.numer, wystawiona.numer);
        assert_eq!(wplaty_faktury(&conn, id).unwrap().len(), 1);
    }

    #[test]
    fn korekta_tworzy_pozycje_przed_i_po() {
        let mut conn = baza();
        let id = prosta_faktura(&mut conn);
        wystaw_fakture(&mut conn, id).unwrap();
        let korekta_id = przygotuj_korekte(&mut conn, id).unwrap();
        let korekta = pobierz_fakture(&conn, korekta_id).unwrap().unwrap();
        assert_eq!(korekta.rodzaj, RodzajFaktury::KorektaSprzedazy);
        assert_eq!(korekta.faktura_korygowana_id, Some(id));
        let bazowa = pobierz_fakture(&conn, id).unwrap().unwrap();
        assert_eq!(bazowa.faktura_korygujaca_id, Some(korekta_id));
        let pozycje = pozycje_faktury(&conn, korekta_id).unwrap();
        assert_eq!(pozycje.len(), 2);
        let przed = pozycje.iter().find(|p| p.czy_przed_korekta).unwrap();
        let po = pozycje.iter().find(|p| !p.czy_przed_korekta).unwrap();
        assert_eq!(przed.wartosc_netto, d!(-200.00));
        assert_eq!(po.wartosc_netto, d!(200.00));
    }

    #[test]
    fn podobna_kopiuje_pozycje_bez_numeru() {
        let mut conn = baza();
        let id = prosta_faktura(&mut conn);
        wystaw_fakture(&mut conn, id).unwrap();
        let kopia_id = przygotuj_podobna(&mut conn, id).unwrap();
        let kopia = pobierz_fakture(&conn, kopia_id).unwrap().unwrap();
        assert_eq!(kopia.numer, "");
        assert_eq!(kopia.razem_brutto, d!(246.00));
        assert_eq!(pozycje_faktury(&conn, kopia_id).unwrap().len(), 1);
    }

    #[test]
    fn lista_faktur_filtruje_i_liczy_wplaty() {
        let mut conn = baza();
        let id = prosta_faktura(&mut conn);
        dodaj_wplate(
            &conn,
            &Wplata {
                id: 0,
                faktura_id: id,
                data: chrono::Local::now().date_naive(),
                kwota: d!(100),
                uwagi: String::new(),
                czy_rozliczenie: false,
            },
        )
        .unwrap();
        let sprzedaz = lista_faktur(&conn, Some(true)).unwrap();
        assert_eq!(sprzedaz.len(), 1);
        assert_eq!(sprzedaz[0].suma_wplat, d!(100));
        assert_eq!(sprzedaz[0].pozostalo_do_zaplaty, d!(146.00));
        assert_eq!(sprzedaz[0].waluta_skrot, "");
        let zakup = lista_faktur(&conn, Some(false)).unwrap();
        assert!(zakup.is_empty());
    }
}
