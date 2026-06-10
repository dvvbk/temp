//! Otwieranie bazy, schemat i dane startowe (odpowiednik Baza + DaneStartowe).

use rusqlite::Connection;
use std::path::Path;

use crate::Wynik;

pub fn otworz(sciezka: &Path) -> Wynik<Connection> {
    let conn = Connection::open(sciezka)?;
    przygotuj(&conn)?;
    Ok(conn)
}

pub fn otworz_w_pamieci() -> Wynik<Connection> {
    let conn = Connection::open_in_memory()?;
    przygotuj(&conn)?;
    Ok(conn)
}

fn przygotuj(conn: &Connection) -> Wynik<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    utworz_schemat(conn)?;
    zaladuj_dane_startowe(conn)?;
    Ok(())
}

fn utworz_schemat(conn: &Connection) -> Wynik<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS kontrahent (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nazwa TEXT NOT NULL DEFAULT '',
            pelna_nazwa TEXT NOT NULL DEFAULT '',
            nip TEXT NOT NULL DEFAULT '',
            adres_rejestrowy TEXT NOT NULL DEFAULT '',
            adres_korespondencyjny TEXT NOT NULL DEFAULT '',
            rachunek_bankowy TEXT NOT NULL DEFAULT '',
            nazwa_banku TEXT NOT NULL DEFAULT '',
            telefon TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            uwagi_wewnetrzne TEXT NOT NULL DEFAULT '',
            uwagi_publiczne TEXT NOT NULL DEFAULT '',
            czy_archiwalny INTEGER NOT NULL DEFAULT 0,
            czy_podmiot INTEGER NOT NULL DEFAULT 0,
            czy_tp INTEGER NOT NULL DEFAULT 0,
            sposob_platnosci_id INTEGER REFERENCES sposob_platnosci(id),
            domyslna_waluta_id INTEGER REFERENCES waluta(id)
        );

        CREATE TABLE IF NOT EXISTS stawka_vat (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            skrot TEXT NOT NULL DEFAULT '',
            wartosc TEXT NOT NULL DEFAULT '0',
            czy_domyslna INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS jednostka_miary (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            skrot TEXT NOT NULL DEFAULT '',
            nazwa TEXT NOT NULL DEFAULT '',
            czy_domyslna INTEGER NOT NULL DEFAULT 0,
            liczba_miejsc_po_przecinku INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS waluta (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            skrot TEXT NOT NULL DEFAULT '',
            nazwa TEXT NOT NULL DEFAULT '',
            czy_domyslna INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS sposob_platnosci (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nazwa TEXT NOT NULL DEFAULT '',
            liczba_dni INTEGER NOT NULL DEFAULT 0,
            czy_domyslny INTEGER NOT NULL DEFAULT 0,
            czy_zaplacone INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS towar (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nazwa TEXT NOT NULL DEFAULT '',
            rodzaj TEXT NOT NULL DEFAULT 'Towar',
            cena_netto TEXT NOT NULL DEFAULT '0',
            cena_brutto TEXT NOT NULL DEFAULT '0',
            sposob_liczenia_ceny TEXT NOT NULL DEFAULT 'WedługNetto',
            czy_archiwalny INTEGER NOT NULL DEFAULT 0,
            gtu INTEGER NOT NULL DEFAULT 0,
            stawka_ryczaltu TEXT,
            stawka_vat_id INTEGER REFERENCES stawka_vat(id),
            jednostka_miary_id INTEGER REFERENCES jednostka_miary(id)
        );

        CREATE TABLE IF NOT EXISTS numerator (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            przeznaczenie TEXT NOT NULL DEFAULT 'Faktura',
            format TEXT NOT NULL DEFAULT '[Numer]',
            grupa TEXT
        );

        CREATE TABLE IF NOT EXISTS stan_numeratora (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            numerator_id INTEGER NOT NULL REFERENCES numerator(id) ON DELETE CASCADE,
            parametry TEXT NOT NULL DEFAULT '',
            ostatnia_wartosc INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS faktura (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            numer TEXT NOT NULL DEFAULT '',
            data_wystawienia TEXT NOT NULL,
            data_sprzedazy TEXT NOT NULL,
            data_wprowadzenia TEXT NOT NULL,
            termin_platnosci TEXT NOT NULL,
            nip_sprzedawcy TEXT NOT NULL DEFAULT '',
            nazwa_sprzedawcy TEXT NOT NULL DEFAULT '',
            dane_sprzedawcy TEXT NOT NULL DEFAULT '',
            nip_nabywcy TEXT NOT NULL DEFAULT '',
            nazwa_nabywcy TEXT NOT NULL DEFAULT '',
            dane_nabywcy TEXT NOT NULL DEFAULT '',
            rachunek_bankowy TEXT NOT NULL DEFAULT '',
            nazwa_banku TEXT NOT NULL DEFAULT '',
            uwagi_publiczne TEXT NOT NULL DEFAULT '',
            uwagi_wewnetrzne TEXT NOT NULL DEFAULT '',
            razem_netto TEXT NOT NULL DEFAULT '0',
            razem_vat TEXT NOT NULL DEFAULT '0',
            razem_brutto TEXT NOT NULL DEFAULT '0',
            kurs_waluty TEXT NOT NULL DEFAULT '0',
            opis_sposobu_platnosci TEXT NOT NULL DEFAULT '',
            rodzaj TEXT NOT NULL DEFAULT 'Sprzedaż',
            czy_wartosci_reczne INTEGER NOT NULL DEFAULT 0,
            procedura_marzy TEXT NOT NULL DEFAULT 'NieDotyczy',
            numer_ksef TEXT NOT NULL DEFAULT '',
            sprzedawca_id INTEGER REFERENCES kontrahent(id),
            nabywca_id INTEGER REFERENCES kontrahent(id),
            faktura_korygowana_id INTEGER REFERENCES faktura(id),
            faktura_korygujaca_id INTEGER REFERENCES faktura(id),
            waluta_id INTEGER REFERENCES waluta(id),
            sposob_platnosci_id INTEGER REFERENCES sposob_platnosci(id)
        );

        CREATE TABLE IF NOT EXISTS pozycja_faktury (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            faktura_id INTEGER NOT NULL REFERENCES faktura(id) ON DELETE CASCADE,
            towar_id INTEGER REFERENCES towar(id),
            opis TEXT NOT NULL DEFAULT '',
            cena_netto TEXT NOT NULL DEFAULT '0',
            cena_vat TEXT NOT NULL DEFAULT '0',
            cena_brutto TEXT NOT NULL DEFAULT '0',
            ilosc TEXT NOT NULL DEFAULT '1',
            wartosc_netto TEXT NOT NULL DEFAULT '0',
            wartosc_vat TEXT NOT NULL DEFAULT '0',
            wartosc_brutto TEXT NOT NULL DEFAULT '0',
            czy_wedlug_cen_brutto INTEGER NOT NULL DEFAULT 0,
            czy_wartosci_reczne INTEGER NOT NULL DEFAULT 0,
            stawka_vat_id INTEGER REFERENCES stawka_vat(id),
            jednostka_miary_id INTEGER REFERENCES jednostka_miary(id),
            lp INTEGER NOT NULL DEFAULT 0,
            czy_przed_korekta INTEGER NOT NULL DEFAULT 0,
            gtu INTEGER NOT NULL DEFAULT 0,
            stawka_ryczaltu TEXT,
            rabat_procent TEXT NOT NULL DEFAULT '0',
            rabat_cena TEXT NOT NULL DEFAULT '0',
            rabat_wartosc TEXT NOT NULL DEFAULT '0',
            cena_zakupu_dla_marzy TEXT NOT NULL DEFAULT '0'
        );

        CREATE TABLE IF NOT EXISTS wplata (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            faktura_id INTEGER NOT NULL REFERENCES faktura(id) ON DELETE CASCADE,
            data TEXT NOT NULL,
            kwota TEXT NOT NULL DEFAULT '0',
            uwagi TEXT NOT NULL DEFAULT '',
            czy_rozliczenie INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS konfiguracja (
            klucz TEXT PRIMARY KEY,
            wartosc TEXT NOT NULL DEFAULT ''
        );

        CREATE INDEX IF NOT EXISTS idx_pozycja_faktura ON pozycja_faktury(faktura_id);
        CREATE INDEX IF NOT EXISTS idx_wplata_faktura ON wplata(faktura_id);
        CREATE INDEX IF NOT EXISTS idx_faktura_rodzaj ON faktura(rodzaj);
        "#,
    )?;
    Ok(())
}

/// Odpowiednik DaneStartowe.Zaladuj - domyślne słowniki przy pierwszym uruchomieniu.
fn zaladuj_dane_startowe(conn: &Connection) -> Wynik<()> {
    let liczba: i64 = conn.query_row("SELECT COUNT(*) FROM jednostka_miary", [], |r| r.get(0))?;
    if liczba == 0 {
        conn.execute_batch(
            r#"
            INSERT INTO jednostka_miary (skrot, nazwa, czy_domyslna, liczba_miejsc_po_przecinku) VALUES
                ('szt', 'Sztuka', 1, 0),
                ('kpl', 'Komplet', 0, 0),
                ('h', 'Godzina', 0, 0),
                ('kg', 'Kilogram', 0, 3),
                ('l', 'Litr', 0, 3);
            "#,
        )?;
    }

    let liczba: i64 = conn.query_row("SELECT COUNT(*) FROM numerator", [], |r| r.get(0))?;
    if liczba == 0 {
        conn.execute_batch(
            r#"
            INSERT INTO numerator (przeznaczenie, format) VALUES
                ('Faktura', 'FV/[Numer]/[Rok]'),
                ('KorektaSprzedaży', 'FK/[Numer]/[Rok]'),
                ('Proforma', 'FP/[Numer]/[Rok]'),
                ('DowódWewnętrzny', 'DW/[Numer]/[Rok]'),
                ('VatMarża', 'FM/[Numer]/[Rok]'),
                ('KorektaVatMarży', 'FKM/[Numer]/[Rok]'),
                ('Rachunek', 'R/[Numer]/[Rok]'),
                ('KorektaRachunku', 'RK/[Numer]/[Rok]');
            "#,
        )?;
    }

    let liczba: i64 = conn.query_row("SELECT COUNT(*) FROM sposob_platnosci", [], |r| r.get(0))?;
    if liczba == 0 {
        conn.execute_batch(
            r#"
            INSERT INTO sposob_platnosci (nazwa, liczba_dni, czy_domyslny, czy_zaplacone) VALUES
                ('Przelew 7', 7, 1, 0),
                ('Przelew 14', 14, 0, 0),
                ('Przelew 30', 30, 0, 0),
                ('Przelew, mechanizm podzielonej płatności', 30, 0, 0),
                ('Gotówka', 0, 0, 1),
                ('Karta', 0, 0, 1);
            "#,
        )?;
    }

    let liczba: i64 = conn.query_row("SELECT COUNT(*) FROM stawka_vat", [], |r| r.get(0))?;
    if liczba == 0 {
        conn.execute_batch(
            r#"
            INSERT INTO stawka_vat (skrot, wartosc, czy_domyslna) VALUES
                ('23%', '23', 1),
                ('8%', '8', 0),
                ('5%', '5', 0),
                ('0%', '0', 0),
                ('NP', '0', 0),
                ('ZW', '0', 0);
            "#,
        )?;
    }

    let liczba: i64 = conn.query_row("SELECT COUNT(*) FROM waluta", [], |r| r.get(0))?;
    if liczba == 0 {
        conn.execute(
            "INSERT INTO waluta (skrot, nazwa, czy_domyslna) VALUES ('PLN', 'Polski złoty', 1)",
            [],
        )?;
    }

    Ok(())
}
