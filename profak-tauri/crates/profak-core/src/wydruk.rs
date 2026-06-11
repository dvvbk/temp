//! Wydruk faktury do PDF - odwzorowanie układu z ProFak.Wydruki.QuestPDF.Faktura:
//! nagłówek sprzedawca/nabywca + rodzaj/numer/daty, specyfikacja pozycji,
//! podsumowanie według stawek VAT, stopka z kwotą słownie i danymi płatności.

use std::collections::HashMap;

use genpdf::elements::{Break, FrameCellDecorator, LinearLayout, Paragraph, TableLayout};
use genpdf::{fonts, style, Alignment, Document, Element, SimplePageDecorator};
use rust_decimal::Decimal;
use rusqlite::Connection;

use crate::model::{Faktura, PozycjaFaktury, RodzajFaktury};
use crate::slownie::slownie_kwota;
use crate::{repo, BladProFak, Wynik};

const CZCIONKA: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
const CZCIONKA_GRUBA: &[u8] = include_bytes!("../fonts/DejaVuSans-Bold.ttf");

pub struct WydrukFaktury {
    pub nazwa_pliku: String,
    pub pdf: Vec<u8>,
}

fn blad(e: impl std::fmt::Display) -> BladProFak {
    BladProFak::Logika(format!("Błąd wydruku: {e}"))
}

/// Formatuje kwotę w stylu "1 234,56".
fn kwota(wartosc: Decimal) -> String {
    let tekst = format!("{:.2}", wartosc);
    let (calosc, grosze) = tekst.split_once('.').unwrap_or((tekst.as_str(), "00"));
    let (znak, cyfry) = calosc.strip_prefix('-').map_or(("", calosc), |c| ("-", c));
    let mut wynik = String::new();
    for (i, c) in cyfry.chars().enumerate() {
        if i > 0 && (cyfry.len() - i) % 3 == 0 {
            wynik.push('\u{a0}');
        }
        wynik.push(c);
    }
    format!("{znak}{wynik},{grosze}")
}

fn ilosc_fmt(ilosc: Decimal) -> String {
    ilosc.normalize().to_string().replace('.', ",")
}

fn naglowek_z_trescia(naglowek: &str, tresc: &str) -> Paragraph {
    let mut p = Paragraph::default();
    p.push_styled(format!("{naglowek}: "), style::Style::new().bold());
    p.push(tresc.to_string());
    p
}

fn rodzaj_fmt(rodzaj: RodzajFaktury) -> &'static str {
    match rodzaj {
        RodzajFaktury::Sprzedaz | RodzajFaktury::Zakup => "Faktura",
        RodzajFaktury::KorektaSprzedazy | RodzajFaktury::KorektaZakupu => "Faktura korygująca",
        RodzajFaktury::Proforma => "Faktura proforma",
        RodzajFaktury::DowodWewnetrzny => "Dowód wewnętrzny",
        RodzajFaktury::VatMarza => "Faktura VAT marża",
        RodzajFaktury::KorektaVatMarzy => "Korekta faktury VAT marża",
        RodzajFaktury::Rachunek => "Rachunek",
        RodzajFaktury::KorektaRachunku => "Korekta rachunku",
        RodzajFaktury::Usunieta => "Faktura (usunięta)",
    }
}

pub fn wydrukuj_fakture(conn: &Connection, faktura_id: i64) -> Wynik<WydrukFaktury> {
    let faktura = repo::pobierz_fakture(conn, faktura_id)?
        .ok_or_else(|| BladProFak::Logika("Nie znaleziono faktury.".into()))?;
    let pozycje = repo::pozycje_faktury(conn, faktura_id)?;
    let wplaty = repo::wplaty_faktury(conn, faktura_id)?;

    let stawki: HashMap<i64, String> = repo::lista_stawek_vat(conn)?
        .into_iter()
        .map(|s| (s.id, s.skrot))
        .collect();
    let jednostki: HashMap<i64, String> = repo::lista_jednostek(conn)?
        .into_iter()
        .map(|j| (j.id, j.skrot))
        .collect();
    let waluta_skrot = faktura
        .waluta_id
        .and_then(|id| {
            repo::lista_walut(conn)
                .ok()?
                .into_iter()
                .find(|w| w.id == id)
                .map(|w| w.skrot)
        })
        .unwrap_or_else(|| "PLN".to_string());
    let waluta = if waluta_skrot == "PLN" { "zł".to_string() } else { waluta_skrot };

    let zaplacono: Decimal = wplaty.iter().map(|w| w.kwota).sum();
    let do_zaplaty = (faktura.razem_brutto - zaplacono).max(Decimal::ZERO);

    let pdf = zbuduj_pdf(&faktura, &pozycje, &stawki, &jednostki, &waluta, zaplacono, do_zaplaty)?;

    let nazwa = if faktura.numer.is_empty() {
        format!("faktura-robocza-{}", faktura.id)
    } else {
        faktura.numer.replace(['/', '\\'], "-")
    };
    Ok(WydrukFaktury { nazwa_pliku: format!("{nazwa}.pdf"), pdf })
}

#[allow(clippy::too_many_arguments)]
fn zbuduj_pdf(
    faktura: &Faktura,
    pozycje: &[PozycjaFaktury],
    stawki: &HashMap<i64, String>,
    jednostki: &HashMap<i64, String>,
    waluta: &str,
    zaplacono: Decimal,
    do_zaplaty: Decimal,
) -> Wynik<Vec<u8>> {
    let regular = fonts::FontData::new(CZCIONKA.to_vec(), None).map_err(blad)?;
    let bold = fonts::FontData::new(CZCIONKA_GRUBA.to_vec(), None).map_err(blad)?;
    let rodzina = fonts::FontFamily {
        regular: regular.clone(),
        bold: bold.clone(),
        italic: regular,
        bold_italic: bold,
    };

    let mut dokument = Document::new(rodzina);
    dokument.set_title(format!("{} {}", rodzaj_fmt(faktura.rodzaj), faktura.numer));
    dokument.set_paper_size(genpdf::PaperSize::A4);
    dokument.set_font_size(9);
    dokument.set_line_spacing(1.2);
    let mut dekorator = SimplePageDecorator::new();
    dekorator.set_margins(15);
    dokument.set_page_decorator(dekorator);

    // --- Nagłówek: kontrahenci po lewej, rodzaj/numer/daty po prawej ---
    let mut lewa = LinearLayout::vertical();
    lewa.push(Paragraph::new("Sprzedawca").styled(style::Style::new().bold().with_font_size(11)));
    lewa.push(Paragraph::new(faktura.nazwa_sprzedawcy.clone()));
    for linia in faktura.dane_sprzedawcy.lines() {
        lewa.push(Paragraph::new(linia.to_string()));
    }
    if !faktura.nip_sprzedawcy.is_empty() {
        lewa.push(naglowek_z_trescia("NIP", &faktura.nip_sprzedawcy));
    }
    lewa.push(Break::new(1));
    lewa.push(Paragraph::new("Nabywca").styled(style::Style::new().bold().with_font_size(11)));
    lewa.push(Paragraph::new(faktura.nazwa_nabywcy.clone()));
    for linia in faktura.dane_nabywcy.lines() {
        lewa.push(Paragraph::new(linia.to_string()));
    }
    if !faktura.nip_nabywcy.is_empty() {
        lewa.push(naglowek_z_trescia("NIP", &faktura.nip_nabywcy));
    }

    let mut prawa = LinearLayout::vertical();
    prawa.push(
        Paragraph::new(rodzaj_fmt(faktura.rodzaj))
            .aligned(Alignment::Right)
            .styled(style::Style::new().bold().with_font_size(13)),
    );
    prawa.push(
        Paragraph::new(faktura.numer.clone())
            .aligned(Alignment::Right)
            .styled(style::Style::new().with_font_size(13)),
    );
    prawa.push(Break::new(1));
    let data_wystawienia = faktura.data_wystawienia.format("%Y-%m-%d").to_string();
    let data_sprzedazy = faktura.data_sprzedazy.format("%Y-%m-%d").to_string();
    {
        let mut p = Paragraph::default();
        p.push_styled("Data wystawienia: ", style::Style::new().bold());
        p.push(data_wystawienia);
        prawa.push(p.aligned(Alignment::Right));
    }
    {
        let mut p = Paragraph::default();
        p.push_styled("Data sprzedaży: ", style::Style::new().bold());
        p.push(data_sprzedazy);
        prawa.push(p.aligned(Alignment::Right));
    }

    let mut naglowek = TableLayout::new(vec![1, 1]);
    let mut wiersz = naglowek.row();
    wiersz.push_element(lewa);
    wiersz.push_element(prawa);
    wiersz.push().map_err(blad)?;
    dokument.push(naglowek);
    dokument.push(Break::new(1.5));

    // --- Specyfikacja pozycji ---
    // Kolumny: LP | Nazwa | Ilość | JM | Cena netto | Wartość netto | VAT | Wartość VAT | Wartość brutto
    let mut tabela = TableLayout::new(vec![3, 22, 5, 4, 8, 9, 5, 8, 9]);
    tabela.set_cell_decorator(FrameCellDecorator::new(true, true, false));

    let styl_naglowka = style::Style::new().bold().with_font_size(8);
    let styl_komorki = style::Style::new().with_font_size(8);

    let komorka = |tekst: String, wyrownanie: Alignment, styl: style::Style| {
        Paragraph::new(tekst).aligned(wyrownanie).styled(styl).padded(1)
    };

    let mut wiersz = tabela.row();
    for (tytul, wyrownanie) in [
        ("LP", Alignment::Center),
        ("Nazwa towaru lub usługi", Alignment::Left),
        ("Ilość", Alignment::Center),
        ("JM", Alignment::Center),
        ("Cena netto", Alignment::Center),
        ("Wartość netto", Alignment::Center),
        ("VAT", Alignment::Center),
        ("Wartość VAT", Alignment::Center),
        ("Wartość brutto", Alignment::Center),
    ] {
        wiersz.push_element(komorka(tytul.to_string(), wyrownanie, styl_naglowka));
    }
    wiersz.push().map_err(blad)?;

    let czy_korekta = pozycje.iter().any(|p| p.czy_przed_korekta);
    let grupy: Vec<(Option<&str>, Vec<&PozycjaFaktury>)> = if czy_korekta {
        vec![
            (Some("Przed korektą"), pozycje.iter().filter(|p| p.czy_przed_korekta).collect()),
            (Some("Po korekcie"), pozycje.iter().filter(|p| !p.czy_przed_korekta).collect()),
        ]
    } else {
        vec![(None, pozycje.iter().collect())]
    };

    for (tytul_grupy, pozycje_grupy) in grupy {
        if let Some(tytul) = tytul_grupy {
            let mut wiersz = tabela.row();
            wiersz.push_element(komorka(String::new(), Alignment::Center, styl_komorki));
            wiersz.push_element(komorka(tytul.to_string(), Alignment::Left, styl_naglowka));
            for _ in 0..7 {
                wiersz.push_element(komorka(String::new(), Alignment::Center, styl_komorki));
            }
            wiersz.push().map_err(blad)?;
        }
        for pozycja in pozycje_grupy {
            let stawka = pozycja
                .stawka_vat_id
                .and_then(|id| stawki.get(&id).cloned())
                .unwrap_or_default();
            let jm = pozycja
                .jednostka_miary_id
                .and_then(|id| jednostki.get(&id).cloned())
                .unwrap_or_default();
            let mut wiersz = tabela.row();
            wiersz.push_element(komorka(pozycja.lp.to_string(), Alignment::Center, styl_komorki));
            wiersz.push_element(komorka(pozycja.opis.clone(), Alignment::Left, styl_komorki));
            wiersz.push_element(komorka(ilosc_fmt(pozycja.ilosc), Alignment::Right, styl_komorki));
            wiersz.push_element(komorka(jm, Alignment::Center, styl_komorki));
            wiersz.push_element(komorka(kwota(pozycja.cena_netto), Alignment::Right, styl_komorki));
            wiersz.push_element(komorka(kwota(pozycja.wartosc_netto), Alignment::Right, styl_komorki));
            wiersz.push_element(komorka(stawka, Alignment::Center, styl_komorki));
            wiersz.push_element(komorka(kwota(pozycja.wartosc_vat), Alignment::Right, styl_komorki));
            wiersz.push_element(komorka(kwota(pozycja.wartosc_brutto), Alignment::Right, styl_komorki));
            wiersz.push().map_err(blad)?;
        }
    }

    // --- Podsumowanie według stawek VAT ---
    let mut sumy_stawek: Vec<(String, Decimal, Decimal, Decimal)> = Vec::new();
    for pozycja in pozycje {
        let stawka = pozycja
            .stawka_vat_id
            .and_then(|id| stawki.get(&id).cloned())
            .unwrap_or_else(|| "-".to_string());
        match sumy_stawek.iter_mut().find(|(s, _, _, _)| *s == stawka) {
            Some(suma) => {
                suma.1 += pozycja.wartosc_netto;
                suma.2 += pozycja.wartosc_vat;
                suma.3 += pozycja.wartosc_brutto;
            }
            None => sumy_stawek.push((stawka, pozycja.wartosc_netto, pozycja.wartosc_vat, pozycja.wartosc_brutto)),
        }
    }

    let pusta = |wiersz: &mut genpdf::elements::TableLayoutRow, ile: usize| {
        for _ in 0..ile {
            wiersz.push_element(Paragraph::new("").padded(1));
        }
    };

    if sumy_stawek.len() > 1 {
        for (stawka, netto, vat, brutto) in &sumy_stawek {
            let mut wiersz = tabela.row();
            pusta(&mut wiersz, 4);
            wiersz.push_element(komorka("W tym".to_string(), Alignment::Right, styl_naglowka));
            wiersz.push_element(komorka(kwota(*netto), Alignment::Right, styl_komorki));
            wiersz.push_element(komorka(stawka.clone(), Alignment::Center, styl_komorki));
            wiersz.push_element(komorka(kwota(*vat), Alignment::Right, styl_komorki));
            wiersz.push_element(komorka(kwota(*brutto), Alignment::Right, styl_komorki));
            wiersz.push().map_err(blad)?;
        }
    }

    let mut wiersz = tabela.row();
    pusta(&mut wiersz, 4);
    wiersz.push_element(komorka("Razem".to_string(), Alignment::Right, styl_naglowka));
    wiersz.push_element(komorka(kwota(faktura.razem_netto), Alignment::Right, styl_naglowka));
    wiersz.push_element(komorka("-".to_string(), Alignment::Center, styl_komorki));
    wiersz.push_element(komorka(kwota(faktura.razem_vat), Alignment::Right, styl_naglowka));
    wiersz.push_element(komorka(kwota(faktura.razem_brutto), Alignment::Right, styl_naglowka));
    wiersz.push().map_err(blad)?;

    dokument.push(tabela);
    dokument.push(Break::new(1.5));

    // --- Stopka ---
    let mut stopka = LinearLayout::vertical();
    if zaplacono > Decimal::ZERO {
        stopka.push(naglowek_z_trescia("Zapłacono", &format!("{} {}", kwota(zaplacono), waluta)));
    }
    stopka.push(naglowek_z_trescia("Do zapłaty", &format!("{} {}", kwota(do_zaplaty), waluta)));
    stopka.push(naglowek_z_trescia("Słownie", &slownie_kwota(do_zaplaty, waluta)));
    stopka.push(naglowek_z_trescia(
        "Termin płatności",
        &faktura.termin_platnosci.format("%Y-%m-%d").to_string(),
    ));
    if !faktura.opis_sposobu_platnosci.is_empty() {
        stopka.push(naglowek_z_trescia("Forma płatności", &faktura.opis_sposobu_platnosci));
    }
    if !faktura.rachunek_bankowy.is_empty() {
        stopka.push(naglowek_z_trescia("Numer rachunku", &faktura.rachunek_bankowy));
    }
    if !faktura.nazwa_banku.is_empty() {
        stopka.push(naglowek_z_trescia("Nazwa banku", &faktura.nazwa_banku));
    }
    if !faktura.uwagi_publiczne.is_empty() {
        stopka.push(Break::new(0.5));
        for linia in faktura.uwagi_publiczne.lines() {
            stopka.push(Paragraph::new(linia.to_string()));
        }
    }
    dokument.push(stopka);

    let mut bufor = Vec::new();
    dokument.render(&mut bufor).map_err(blad)?;
    Ok(bufor)
}

#[cfg(test)]
mod testy {
    use super::*;
    use crate::model::Faktura;
    use rust_decimal_macros::dec;

    #[test]
    fn formatuje_kwoty() {
        assert_eq!(kwota(dec!(1234.5)), "1\u{a0}234,50");
        assert_eq!(kwota(dec!(-1234567.89)), "-1\u{a0}234\u{a0}567,89");
        assert_eq!(kwota(dec!(0)), "0,00");
    }

    #[test]
    fn generuje_pdf_faktury() {
        let mut conn = crate::db::otworz_w_pamieci().unwrap();
        let stawka: i64 = conn
            .query_row("SELECT id FROM stawka_vat WHERE skrot = '23%'", [], |r| r.get(0))
            .unwrap();
        let faktura = Faktura {
            nazwa_sprzedawcy: "Testowy Sprzedawca Sp. z o.o.".into(),
            nip_sprzedawcy: "1234567890".into(),
            dane_sprzedawcy: "ul. Testowa 1\n00-001 Warszawa".into(),
            nazwa_nabywcy: "Klient SA".into(),
            ..Faktura::default()
        };
        let pozycja = PozycjaFaktury {
            opis: "Usługa ze znakami: ąćęłńóśźż".into(),
            cena_netto: dec!(1500),
            ilosc: dec!(2),
            stawka_vat_id: Some(stawka),
            ..PozycjaFaktury::default()
        };
        let id = repo::zapisz_fakture(&mut conn, faktura, vec![pozycja]).unwrap();
        repo::wystaw_fakture(&mut conn, id).unwrap();
        let wydruk = wydrukuj_fakture(&conn, id).unwrap();
        assert!(wydruk.pdf.starts_with(b"%PDF"));
        assert!(wydruk.pdf.len() > 10_000, "PDF podejrzanie mały: {} B", wydruk.pdf.len());
        let rok = chrono::Local::now().format("%Y").to_string();
        assert_eq!(wydruk.nazwa_pliku, format!("FV-1-{rok}.pdf"));
    }

    #[test]
    fn wydruk_korekty_grupuje_pozycje() {
        let mut conn = crate::db::otworz_w_pamieci().unwrap();
        let stawka: i64 = conn
            .query_row("SELECT id FROM stawka_vat WHERE skrot = '8%'", [], |r| r.get(0))
            .unwrap();
        let pozycja = PozycjaFaktury {
            opis: "Towar".into(),
            cena_netto: dec!(100),
            ilosc: dec!(1),
            stawka_vat_id: Some(stawka),
            ..PozycjaFaktury::default()
        };
        let id = repo::zapisz_fakture(&mut conn, Faktura::default(), vec![pozycja]).unwrap();
        repo::wystaw_fakture(&mut conn, id).unwrap();
        let korekta_id = repo::przygotuj_korekte(&mut conn, id).unwrap();
        let wydruk = wydrukuj_fakture(&conn, korekta_id).unwrap();
        assert!(wydruk.pdf.starts_with(b"%PDF"));
    }
}
