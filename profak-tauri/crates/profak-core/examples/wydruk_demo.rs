//! Generuje przykładowy wydruk faktury do pliku PDF (do ręcznej weryfikacji układu).

use profak_core::model::{Faktura, Kontrahent, PozycjaFaktury, Wplata};
use profak_core::{db, repo, wydruk};
use rust_decimal::Decimal;
use std::str::FromStr;

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn main() {
    let mut conn = db::otworz_w_pamieci().unwrap();
    let stawka_23: i64 = conn.query_row("SELECT id FROM stawka_vat WHERE skrot='23%'", [], |r| r.get(0)).unwrap();
    let stawka_8: i64 = conn.query_row("SELECT id FROM stawka_vat WHERE skrot='8%'", [], |r| r.get(0)).unwrap();
    let szt: i64 = conn.query_row("SELECT id FROM jednostka_miary WHERE skrot='szt'", [], |r| r.get(0)).unwrap();
    let h: i64 = conn.query_row("SELECT id FROM jednostka_miary WHERE skrot='h'", [], |r| r.get(0)).unwrap();
    let przelew: i64 = conn.query_row("SELECT id FROM sposob_platnosci WHERE nazwa='Przelew 14'", [], |r| r.get(0)).unwrap();
    let pln: i64 = conn.query_row("SELECT id FROM waluta WHERE skrot='PLN'", [], |r| r.get(0)).unwrap();

    let sprzedawca = Kontrahent {
        nazwa: "Przykładowa Firma".into(),
        pelna_nazwa: "Przykładowa Firma Sp. z o.o.".into(),
        nip: "521-301-72-28".into(),
        adres_rejestrowy: "ul. Łąkowa 15/3\n00-175 Warszawa".into(),
        rachunek_bankowy: "61 1090 1014 0000 0712 1981 2874".into(),
        nazwa_banku: "Santander Bank Polska".into(),
        czy_podmiot: true,
        ..Kontrahent::default()
    };
    let sprzedawca_id = repo::zapisz_kontrahenta(&conn, &sprzedawca).unwrap();

    let mut faktura = Faktura::default();
    faktura.sprzedawca_id = Some(sprzedawca_id);
    faktura.nazwa_sprzedawcy = sprzedawca.pelna_nazwa.clone();
    faktura.nip_sprzedawcy = sprzedawca.nip.clone();
    faktura.dane_sprzedawcy = sprzedawca.adres_rejestrowy.clone();
    faktura.rachunek_bankowy = sprzedawca.rachunek_bankowy.clone();
    faktura.nazwa_banku = sprzedawca.nazwa_banku.clone();
    faktura.nazwa_nabywcy = "Główczyński i Wspólnicy S.K.A.".into();
    faktura.nip_nabywcy = "951-234-56-78".into();
    faktura.dane_nabywcy = "al. Niepodległości 208\n02-086 Warszawa".into();
    faktura.sposob_platnosci_id = Some(przelew);
    faktura.opis_sposobu_platnosci = "Przelew 14".into();
    faktura.waluta_id = Some(pln);
    faktura.termin_platnosci = faktura.data_wystawienia + chrono::Duration::days(14);
    faktura.uwagi_publiczne = "Dziękujemy za terminową płatność.".into();

    let pozycje = vec![
        PozycjaFaktury {
            opis: "Usługi programistyczne — moduł raportów".into(),
            cena_netto: d("180"),
            ilosc: d("42"),
            stawka_vat_id: Some(stawka_23),
            jednostka_miary_id: Some(h),
            ..PozycjaFaktury::default()
        },
        PozycjaFaktury {
            opis: "Licencja roczna ProFak Plus".into(),
            cena_netto: d("1200"),
            ilosc: d("2"),
            rabat_procent: d("10"),
            stawka_vat_id: Some(stawka_23),
            jednostka_miary_id: Some(szt),
            ..PozycjaFaktury::default()
        },
        PozycjaFaktury {
            opis: "Podręcznik użytkownika (druk)".into(),
            cena_netto: d("49.90"),
            ilosc: d("3"),
            stawka_vat_id: Some(stawka_8),
            jednostka_miary_id: Some(szt),
            ..PozycjaFaktury::default()
        },
    ];

    let id = repo::zapisz_fakture(&mut conn, faktura, pozycje).unwrap();
    let wystawiona = repo::wystaw_fakture(&mut conn, id).unwrap();
    repo::dodaj_wplate(&conn, &Wplata {
        id: 0,
        faktura_id: id,
        data: wystawiona.data_wystawienia,
        kwota: d("3000"),
        uwagi: "".into(),
        czy_rozliczenie: false,
    }).unwrap();

    let wydruk = wydruk::wydrukuj_fakture(&conn, id).unwrap();
    std::fs::write(format!("/tmp/{}", wydruk.nazwa_pliku), &wydruk.pdf).unwrap();
    println!("/tmp/{}", wydruk.nazwa_pliku);
}
