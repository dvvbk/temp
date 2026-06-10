//! Wyliczenia faktur - wierne odwzorowanie PozycjaFaktury.PrzeliczCeny
//! i Faktura.PrzeliczRazem z oryginalnego ProFaka.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::liczby::zaokragl;
use crate::model::{Faktura, PozycjaFaktury, RodzajFaktury};

/// Przelicza ceny i wartości pozycji dla podanej stawki VAT (w procentach).
/// `vat_marza` = czy faktura jest typu VatMarża/KorektaVatMarży.
pub fn przelicz_ceny(pozycja: &mut PozycjaFaktury, procent_vat: Decimal, vat_marza: bool) {
    if pozycja.czy_wartosci_reczne {
        return;
    }
    if vat_marza {
        pozycja.czy_wedlug_cen_brutto = true;
    }

    let sto = dec!(100);

    if pozycja.cena_zakupu_dla_marzy > Decimal::ZERO {
        let marza_brutto = pozycja.cena_brutto - pozycja.cena_zakupu_dla_marzy;
        let marza_netto = zaokragl(marza_brutto * sto / (sto + procent_vat));
        pozycja.cena_netto = marza_netto;
        pozycja.cena_vat = zaokragl(marza_brutto - marza_netto);
        pozycja.wartosc_brutto = zaokragl(pozycja.ilosc * pozycja.cena_brutto);
        pozycja.wartosc_netto = zaokragl(pozycja.ilosc * pozycja.cena_netto);
        pozycja.wartosc_vat = zaokragl(pozycja.ilosc * pozycja.cena_vat);
    } else if pozycja.czy_wedlug_cen_brutto {
        pozycja.cena_netto = zaokragl(pozycja.cena_brutto * sto / (sto + procent_vat));
        pozycja.cena_vat = zaokragl(pozycja.cena_brutto - pozycja.cena_netto);
        pozycja.wartosc_brutto = zaokragl(
            pozycja.ilosc * pozycja.cena_brutto * (sto - pozycja.rabat_procent) / sto
                - pozycja.rabat_cena * pozycja.ilosc
                - pozycja.rabat_wartosc,
        );
        if pozycja.wartosc_brutto * znak(pozycja.ilosc) < Decimal::ZERO {
            pozycja.wartosc_brutto = Decimal::ZERO;
        }
        pozycja.wartosc_netto = zaokragl(pozycja.wartosc_brutto * sto / (sto + procent_vat));
        pozycja.wartosc_vat = zaokragl(pozycja.wartosc_brutto - pozycja.wartosc_netto);
    } else {
        pozycja.cena_vat = zaokragl(pozycja.cena_netto * procent_vat / sto);
        pozycja.cena_brutto = zaokragl(pozycja.cena_netto + pozycja.cena_vat);
        pozycja.wartosc_netto = zaokragl(
            pozycja.ilosc * pozycja.cena_netto * (sto - pozycja.rabat_procent) / sto
                - pozycja.rabat_cena * pozycja.ilosc
                - pozycja.rabat_wartosc,
        );
        if pozycja.wartosc_netto * znak(pozycja.ilosc) < Decimal::ZERO {
            pozycja.wartosc_netto = Decimal::ZERO;
        }
        pozycja.wartosc_vat = zaokragl(pozycja.wartosc_netto * procent_vat / sto);
        pozycja.wartosc_brutto = zaokragl(pozycja.wartosc_netto + pozycja.wartosc_vat);
    }
}

fn znak(wartosc: Decimal) -> Decimal {
    if wartosc < Decimal::ZERO {
        dec!(-1)
    } else if wartosc > Decimal::ZERO {
        Decimal::ONE
    } else {
        Decimal::ZERO
    }
}

/// Odpowiednik Faktura.PrzeliczRazem - sumuje wartości pozycji,
/// chyba że wartości faktury są wpisane ręcznie.
pub fn przelicz_razem(faktura: &mut Faktura, pozycje: &[PozycjaFaktury]) {
    if faktura.czy_wartosci_reczne {
        return;
    }
    faktura.razem_netto = pozycje.iter().map(|p| p.wartosc_netto).sum();
    faktura.razem_vat = pozycje.iter().map(|p| p.wartosc_vat).sum();
    faktura.razem_brutto = pozycje.iter().map(|p| p.wartosc_brutto).sum();
}

/// Numeruje pozycje od 1 (z pominięciem pozycji "przed korektą" - jak w oryginale).
pub fn popraw_numeracje_pozycji(rodzaj: RodzajFaktury, pozycje: &mut [PozycjaFaktury]) {
    if matches!(
        rodzaj,
        RodzajFaktury::KorektaZakupu | RodzajFaktury::KorektaSprzedazy | RodzajFaktury::KorektaVatMarzy
    ) {
        return;
    }
    let mut lp = 1;
    for pozycja in pozycje.iter_mut() {
        if pozycja.czy_przed_korekta {
            continue;
        }
        pozycja.lp = lp;
        lp += 1;
    }
}

#[cfg(test)]
mod testy {
    use super::*;
    use crate::model::PozycjaFaktury;

    fn pozycja() -> PozycjaFaktury {
        PozycjaFaktury::default()
    }

    #[test]
    fn liczy_wedlug_cen_netto() {
        let mut p = pozycja();
        p.cena_netto = dec!(100);
        p.ilosc = dec!(2);
        przelicz_ceny(&mut p, dec!(23), false);
        assert_eq!(p.cena_vat, dec!(23.00));
        assert_eq!(p.cena_brutto, dec!(123.00));
        assert_eq!(p.wartosc_netto, dec!(200.00));
        assert_eq!(p.wartosc_vat, dec!(46.00));
        assert_eq!(p.wartosc_brutto, dec!(246.00));
    }

    #[test]
    fn liczy_wedlug_cen_brutto() {
        let mut p = pozycja();
        p.czy_wedlug_cen_brutto = true;
        p.cena_brutto = dec!(123);
        p.ilosc = dec!(1);
        przelicz_ceny(&mut p, dec!(23), false);
        assert_eq!(p.cena_netto, dec!(100.00));
        assert_eq!(p.cena_vat, dec!(23.00));
        assert_eq!(p.wartosc_brutto, dec!(123.00));
        assert_eq!(p.wartosc_netto, dec!(100.00));
        assert_eq!(p.wartosc_vat, dec!(23.00));
    }

    #[test]
    fn liczy_rabat_procentowy_od_netto() {
        let mut p = pozycja();
        p.cena_netto = dec!(100);
        p.ilosc = dec!(1);
        p.rabat_procent = dec!(10);
        przelicz_ceny(&mut p, dec!(23), false);
        assert_eq!(p.wartosc_netto, dec!(90.00));
        assert_eq!(p.wartosc_vat, dec!(20.70));
        assert_eq!(p.wartosc_brutto, dec!(110.70));
    }

    #[test]
    fn liczy_marze_od_ceny_zakupu() {
        let mut p = pozycja();
        p.cena_brutto = dec!(1000);
        p.cena_zakupu_dla_marzy = dec!(800);
        p.ilosc = dec!(1);
        przelicz_ceny(&mut p, dec!(23), true);
        // marża brutto 200, netto = 200*100/123 = 162.60, VAT = 37.40
        assert_eq!(p.cena_netto, dec!(162.60));
        assert_eq!(p.cena_vat, dec!(37.40));
        assert_eq!(p.wartosc_brutto, dec!(1000.00));
    }

    #[test]
    fn ujemna_wartosc_z_rabatu_obcinana_do_zera() {
        let mut p = pozycja();
        p.cena_netto = dec!(10);
        p.ilosc = dec!(1);
        p.rabat_wartosc = dec!(20);
        przelicz_ceny(&mut p, dec!(23), false);
        assert_eq!(p.wartosc_netto, dec!(0));
        assert_eq!(p.wartosc_brutto, dec!(0));
    }

    #[test]
    fn wartosci_reczne_nie_sa_przeliczane() {
        let mut p = pozycja();
        p.czy_wartosci_reczne = true;
        p.cena_netto = dec!(100);
        p.wartosc_netto = dec!(999);
        przelicz_ceny(&mut p, dec!(23), false);
        assert_eq!(p.wartosc_netto, dec!(999));
    }

    #[test]
    fn sumuje_pozycje_do_razem() {
        let mut f = Faktura::default();
        let mut p1 = pozycja();
        p1.cena_netto = dec!(100);
        p1.ilosc = dec!(1);
        przelicz_ceny(&mut p1, dec!(23), false);
        let mut p2 = pozycja();
        p2.cena_netto = dec!(50);
        p2.ilosc = dec!(3);
        przelicz_ceny(&mut p2, dec!(8), false);
        przelicz_razem(&mut f, &[p1, p2]);
        assert_eq!(f.razem_netto, dec!(250.00));
        assert_eq!(f.razem_vat, dec!(35.00));
        assert_eq!(f.razem_brutto, dec!(285.00));
    }
}
