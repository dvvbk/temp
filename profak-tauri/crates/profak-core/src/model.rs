//! Model danych - odpowiednik klas z ProFak.DB.
//! Nazwy pól i typy odwzorowują oryginalny model EF Core (Rekord<T>).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RodzajFaktury {
    #[serde(rename = "Sprzedaż")]
    Sprzedaz,
    Zakup,
    #[serde(rename = "KorektaSprzedaży")]
    KorektaSprzedazy,
    KorektaZakupu,
    Proforma,
    #[serde(rename = "DowódWewnętrzny")]
    DowodWewnetrzny,
    #[serde(rename = "Usunięta")]
    Usunieta,
    #[serde(rename = "VatMarża")]
    VatMarza,
    #[serde(rename = "KorektaVatMarży")]
    KorektaVatMarzy,
    Rachunek,
    KorektaRachunku,
}

impl RodzajFaktury {
    pub fn as_str(&self) -> &'static str {
        match self {
            RodzajFaktury::Sprzedaz => "Sprzedaż",
            RodzajFaktury::Zakup => "Zakup",
            RodzajFaktury::KorektaSprzedazy => "KorektaSprzedaży",
            RodzajFaktury::KorektaZakupu => "KorektaZakupu",
            RodzajFaktury::Proforma => "Proforma",
            RodzajFaktury::DowodWewnetrzny => "DowódWewnętrzny",
            RodzajFaktury::Usunieta => "Usunięta",
            RodzajFaktury::VatMarza => "VatMarża",
            RodzajFaktury::KorektaVatMarzy => "KorektaVatMarży",
            RodzajFaktury::Rachunek => "Rachunek",
            RodzajFaktury::KorektaRachunku => "KorektaRachunku",
        }
    }

    pub fn from_str(s: &str) -> RodzajFaktury {
        match s {
            "Zakup" => RodzajFaktury::Zakup,
            "KorektaSprzedaży" => RodzajFaktury::KorektaSprzedazy,
            "KorektaZakupu" => RodzajFaktury::KorektaZakupu,
            "Proforma" => RodzajFaktury::Proforma,
            "DowódWewnętrzny" => RodzajFaktury::DowodWewnetrzny,
            "Usunięta" => RodzajFaktury::Usunieta,
            "VatMarża" => RodzajFaktury::VatMarza,
            "KorektaVatMarży" => RodzajFaktury::KorektaVatMarzy,
            "Rachunek" => RodzajFaktury::Rachunek,
            "KorektaRachunku" => RodzajFaktury::KorektaRachunku,
            _ => RodzajFaktury::Sprzedaz,
        }
    }

    pub fn czy_sprzedaz(&self) -> bool {
        matches!(
            self,
            RodzajFaktury::Sprzedaz
                | RodzajFaktury::KorektaSprzedazy
                | RodzajFaktury::Proforma
                | RodzajFaktury::VatMarza
                | RodzajFaktury::KorektaVatMarzy
                | RodzajFaktury::Rachunek
                | RodzajFaktury::KorektaRachunku
        )
    }

    pub fn czy_zakup(&self) -> bool {
        !self.czy_sprzedaz()
    }

    /// Przeznaczenie numeratora dla danego rodzaju faktury (None = bez numeracji automatycznej).
    pub fn numerator(&self) -> Option<PrzeznaczenieNumeratora> {
        match self {
            RodzajFaktury::Sprzedaz => Some(PrzeznaczenieNumeratora::Faktura),
            RodzajFaktury::Proforma => Some(PrzeznaczenieNumeratora::Proforma),
            RodzajFaktury::KorektaSprzedazy => Some(PrzeznaczenieNumeratora::KorektaSprzedazy),
            RodzajFaktury::DowodWewnetrzny => Some(PrzeznaczenieNumeratora::DowodWewnetrzny),
            RodzajFaktury::VatMarza => Some(PrzeznaczenieNumeratora::VatMarza),
            RodzajFaktury::KorektaVatMarzy => Some(PrzeznaczenieNumeratora::KorektaVatMarzy),
            RodzajFaktury::Rachunek => Some(PrzeznaczenieNumeratora::Rachunek),
            RodzajFaktury::KorektaRachunku => Some(PrzeznaczenieNumeratora::KorektaRachunku),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProceduraMarzy {
    NieDotyczy,
    #[serde(rename = "TowaryUżywane")]
    TowaryUzywane,
    #[serde(rename = "DziełaSztuki")]
    DzielaSztuki,
    #[serde(rename = "BiuraPodróży")]
    BiuraPodrozy,
    PrzedmiotyKolekcjonerskie,
}

impl ProceduraMarzy {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProceduraMarzy::NieDotyczy => "NieDotyczy",
            ProceduraMarzy::TowaryUzywane => "TowaryUżywane",
            ProceduraMarzy::DzielaSztuki => "DziełaSztuki",
            ProceduraMarzy::BiuraPodrozy => "BiuraPodróży",
            ProceduraMarzy::PrzedmiotyKolekcjonerskie => "PrzedmiotyKolekcjonerskie",
        }
    }

    pub fn from_str(s: &str) -> ProceduraMarzy {
        match s {
            "TowaryUżywane" => ProceduraMarzy::TowaryUzywane,
            "DziełaSztuki" => ProceduraMarzy::DzielaSztuki,
            "BiuraPodróży" => ProceduraMarzy::BiuraPodrozy,
            "PrzedmiotyKolekcjonerskie" => ProceduraMarzy::PrzedmiotyKolekcjonerskie,
            _ => ProceduraMarzy::NieDotyczy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrzeznaczenieNumeratora {
    Faktura,
    Proforma,
    #[serde(rename = "KorektaSprzedaży")]
    KorektaSprzedazy,
    #[serde(rename = "DowódWewnętrzny")]
    DowodWewnetrzny,
    #[serde(rename = "VatMarża")]
    VatMarza,
    #[serde(rename = "KorektaVatMarży")]
    KorektaVatMarzy,
    Rachunek,
    KorektaRachunku,
}

impl PrzeznaczenieNumeratora {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrzeznaczenieNumeratora::Faktura => "Faktura",
            PrzeznaczenieNumeratora::Proforma => "Proforma",
            PrzeznaczenieNumeratora::KorektaSprzedazy => "KorektaSprzedaży",
            PrzeznaczenieNumeratora::DowodWewnetrzny => "DowódWewnętrzny",
            PrzeznaczenieNumeratora::VatMarza => "VatMarża",
            PrzeznaczenieNumeratora::KorektaVatMarzy => "KorektaVatMarży",
            PrzeznaczenieNumeratora::Rachunek => "Rachunek",
            PrzeznaczenieNumeratora::KorektaRachunku => "KorektaRachunku",
        }
    }

    pub fn from_str(s: &str) -> PrzeznaczenieNumeratora {
        match s {
            "Proforma" => PrzeznaczenieNumeratora::Proforma,
            "KorektaSprzedaży" => PrzeznaczenieNumeratora::KorektaSprzedazy,
            "DowódWewnętrzny" => PrzeznaczenieNumeratora::DowodWewnetrzny,
            "VatMarża" => PrzeznaczenieNumeratora::VatMarza,
            "KorektaVatMarży" => PrzeznaczenieNumeratora::KorektaVatMarzy,
            "Rachunek" => PrzeznaczenieNumeratora::Rachunek,
            "KorektaRachunku" => PrzeznaczenieNumeratora::KorektaRachunku,
            _ => PrzeznaczenieNumeratora::Faktura,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RodzajTowaru {
    Towar,
    #[serde(rename = "Usługa")]
    Usluga,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SposobLiczeniaCenyTowaru {
    #[serde(rename = "WedługNetto")]
    WedlugNetto,
    #[serde(rename = "WedługBrutto")]
    WedlugBrutto,
    NarzutKwotowy,
    NarzutProcentowy,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Kontrahent {
    pub id: i64,
    pub nazwa: String,
    pub pelna_nazwa: String,
    pub nip: String,
    pub adres_rejestrowy: String,
    pub adres_korespondencyjny: String,
    pub rachunek_bankowy: String,
    pub nazwa_banku: String,
    pub telefon: String,
    pub email: String,
    pub uwagi_wewnetrzne: String,
    pub uwagi_publiczne: String,
    pub czy_archiwalny: bool,
    /// Czy kontrahent jest podmiotem prowadzącym księgowość (sprzedawcą na fakturach sprzedaży).
    pub czy_podmiot: bool,
    pub czy_tp: bool,
    pub sposob_platnosci_id: Option<i64>,
    pub domyslna_waluta_id: Option<i64>,
}

impl Kontrahent {
    pub fn pelna_nazwa_lub_nazwa(&self) -> &str {
        if self.pelna_nazwa.is_empty() {
            &self.nazwa
        } else {
            &self.pelna_nazwa
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Towar {
    pub id: i64,
    pub nazwa: String,
    pub rodzaj: RodzajTowaru,
    pub cena_netto: Decimal,
    pub cena_brutto: Decimal,
    pub sposob_liczenia_ceny: SposobLiczeniaCenyTowaru,
    pub czy_archiwalny: bool,
    pub gtu: i64,
    pub stawka_ryczaltu: Option<Decimal>,
    pub stawka_vat_id: Option<i64>,
    pub jednostka_miary_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StawkaVat {
    pub id: i64,
    pub skrot: String,
    pub wartosc: Decimal,
    pub czy_domyslna: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JednostkaMiary {
    pub id: i64,
    pub skrot: String,
    pub nazwa: String,
    pub czy_domyslna: bool,
    pub liczba_miejsc_po_przecinku: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waluta {
    pub id: i64,
    pub skrot: String,
    pub nazwa: String,
    pub czy_domyslna: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SposobPlatnosci {
    pub id: i64,
    pub nazwa: String,
    pub liczba_dni: i64,
    pub czy_domyslny: bool,
    pub czy_zaplacone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Numerator {
    pub id: i64,
    pub przeznaczenie: PrzeznaczenieNumeratora,
    pub format: String,
    pub grupa: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StanNumeratora {
    pub id: i64,
    pub numerator_id: i64,
    pub parametry: String,
    pub ostatnia_wartosc: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faktura {
    pub id: i64,
    pub numer: String,
    pub data_wystawienia: NaiveDate,
    pub data_sprzedazy: NaiveDate,
    pub data_wprowadzenia: NaiveDate,
    pub termin_platnosci: NaiveDate,
    pub nip_sprzedawcy: String,
    pub nazwa_sprzedawcy: String,
    pub dane_sprzedawcy: String,
    pub nip_nabywcy: String,
    pub nazwa_nabywcy: String,
    pub dane_nabywcy: String,
    pub rachunek_bankowy: String,
    pub nazwa_banku: String,
    pub uwagi_publiczne: String,
    pub uwagi_wewnetrzne: String,
    pub razem_netto: Decimal,
    pub razem_vat: Decimal,
    pub razem_brutto: Decimal,
    pub kurs_waluty: Decimal,
    pub opis_sposobu_platnosci: String,
    pub rodzaj: RodzajFaktury,
    pub czy_wartosci_reczne: bool,
    pub procedura_marzy: ProceduraMarzy,
    pub numer_ksef: String,
    pub sprzedawca_id: Option<i64>,
    pub nabywca_id: Option<i64>,
    pub faktura_korygowana_id: Option<i64>,
    pub faktura_korygujaca_id: Option<i64>,
    pub waluta_id: Option<i64>,
    pub sposob_platnosci_id: Option<i64>,
}

impl Default for Faktura {
    fn default() -> Self {
        let dzis = chrono::Local::now().date_naive();
        Faktura {
            id: 0,
            numer: String::new(),
            data_wystawienia: dzis,
            data_sprzedazy: dzis,
            data_wprowadzenia: dzis,
            termin_platnosci: dzis,
            nip_sprzedawcy: String::new(),
            nazwa_sprzedawcy: String::new(),
            dane_sprzedawcy: String::new(),
            nip_nabywcy: String::new(),
            nazwa_nabywcy: String::new(),
            dane_nabywcy: String::new(),
            rachunek_bankowy: String::new(),
            nazwa_banku: String::new(),
            uwagi_publiczne: String::new(),
            uwagi_wewnetrzne: String::new(),
            razem_netto: Decimal::ZERO,
            razem_vat: Decimal::ZERO,
            razem_brutto: Decimal::ZERO,
            kurs_waluty: Decimal::ZERO,
            opis_sposobu_platnosci: String::new(),
            rodzaj: RodzajFaktury::Sprzedaz,
            czy_wartosci_reczne: false,
            procedura_marzy: ProceduraMarzy::NieDotyczy,
            numer_ksef: String::new(),
            sprzedawca_id: None,
            nabywca_id: None,
            faktura_korygowana_id: None,
            faktura_korygujaca_id: None,
            waluta_id: None,
            sposob_platnosci_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PozycjaFaktury {
    pub id: i64,
    pub faktura_id: i64,
    pub towar_id: Option<i64>,
    pub opis: String,
    pub cena_netto: Decimal,
    pub cena_vat: Decimal,
    pub cena_brutto: Decimal,
    pub ilosc: Decimal,
    pub wartosc_netto: Decimal,
    pub wartosc_vat: Decimal,
    pub wartosc_brutto: Decimal,
    pub czy_wedlug_cen_brutto: bool,
    pub czy_wartosci_reczne: bool,
    pub stawka_vat_id: Option<i64>,
    pub jednostka_miary_id: Option<i64>,
    pub lp: i64,
    pub czy_przed_korekta: bool,
    pub gtu: i64,
    pub stawka_ryczaltu: Option<Decimal>,
    pub rabat_procent: Decimal,
    pub rabat_cena: Decimal,
    pub rabat_wartosc: Decimal,
    pub cena_zakupu_dla_marzy: Decimal,
}

impl Default for PozycjaFaktury {
    fn default() -> Self {
        PozycjaFaktury {
            id: 0,
            faktura_id: 0,
            towar_id: None,
            opis: String::new(),
            cena_netto: Decimal::ZERO,
            cena_vat: Decimal::ZERO,
            cena_brutto: Decimal::ZERO,
            ilosc: Decimal::ONE,
            wartosc_netto: Decimal::ZERO,
            wartosc_vat: Decimal::ZERO,
            wartosc_brutto: Decimal::ZERO,
            czy_wedlug_cen_brutto: false,
            czy_wartosci_reczne: false,
            stawka_vat_id: None,
            jednostka_miary_id: None,
            lp: 0,
            czy_przed_korekta: false,
            gtu: 0,
            stawka_ryczaltu: None,
            rabat_procent: Decimal::ZERO,
            rabat_cena: Decimal::ZERO,
            rabat_wartosc: Decimal::ZERO,
            cena_zakupu_dla_marzy: Decimal::ZERO,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wplata {
    pub id: i64,
    pub faktura_id: i64,
    pub data: NaiveDate,
    pub kwota: Decimal,
    pub uwagi: String,
    pub czy_rozliczenie: bool,
}

/// Wiersz listy faktur z danymi wyliczonymi (suma wpłat itp.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FakturaListaWiersz {
    #[serde(flatten)]
    pub faktura: Faktura,
    pub suma_wplat: Decimal,
    pub pozostalo_do_zaplaty: Decimal,
    pub waluta_skrot: String,
}
