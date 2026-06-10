// Typy i wywołania komend Tauri - odwzorowanie modeli z profak-core.
// Kwoty (Decimal) są serializowane jako stringi, np. "123.45".

import { invoke } from "@tauri-apps/api/core";

export type RodzajFaktury =
  | "Sprzedaż"
  | "Zakup"
  | "KorektaSprzedaży"
  | "KorektaZakupu"
  | "Proforma"
  | "DowódWewnętrzny"
  | "Usunięta"
  | "VatMarża"
  | "KorektaVatMarży"
  | "Rachunek"
  | "KorektaRachunku";

export type ProceduraMarzy =
  | "NieDotyczy"
  | "TowaryUżywane"
  | "DziełaSztuki"
  | "BiuraPodróży"
  | "PrzedmiotyKolekcjonerskie";

export type PrzeznaczenieNumeratora =
  | "Faktura"
  | "Proforma"
  | "KorektaSprzedaży"
  | "DowódWewnętrzny"
  | "VatMarża"
  | "KorektaVatMarży"
  | "Rachunek"
  | "KorektaRachunku";

export interface Kontrahent {
  id: number;
  nazwa: string;
  pelna_nazwa: string;
  nip: string;
  adres_rejestrowy: string;
  adres_korespondencyjny: string;
  rachunek_bankowy: string;
  nazwa_banku: string;
  telefon: string;
  email: string;
  uwagi_wewnetrzne: string;
  uwagi_publiczne: string;
  czy_archiwalny: boolean;
  czy_podmiot: boolean;
  czy_tp: boolean;
  sposob_platnosci_id: number | null;
  domyslna_waluta_id: number | null;
}

export const nowyKontrahent = (): Kontrahent => ({
  id: 0,
  nazwa: "",
  pelna_nazwa: "",
  nip: "",
  adres_rejestrowy: "",
  adres_korespondencyjny: "",
  rachunek_bankowy: "",
  nazwa_banku: "",
  telefon: "",
  email: "",
  uwagi_wewnetrzne: "",
  uwagi_publiczne: "",
  czy_archiwalny: false,
  czy_podmiot: false,
  czy_tp: false,
  sposob_platnosci_id: null,
  domyslna_waluta_id: null,
});

export interface Towar {
  id: number;
  nazwa: string;
  rodzaj: "Towar" | "Usługa";
  cena_netto: string;
  cena_brutto: string;
  sposob_liczenia_ceny: "WedługNetto" | "WedługBrutto" | "NarzutKwotowy" | "NarzutProcentowy";
  czy_archiwalny: boolean;
  gtu: number;
  stawka_ryczaltu: string | null;
  stawka_vat_id: number | null;
  jednostka_miary_id: number | null;
}

export const nowyTowar = (): Towar => ({
  id: 0,
  nazwa: "",
  rodzaj: "Towar",
  cena_netto: "0",
  cena_brutto: "0",
  sposob_liczenia_ceny: "WedługNetto",
  czy_archiwalny: false,
  gtu: 0,
  stawka_ryczaltu: null,
  stawka_vat_id: null,
  jednostka_miary_id: null,
});

export interface StawkaVat {
  id: number;
  skrot: string;
  wartosc: string;
  czy_domyslna: boolean;
}

export interface JednostkaMiary {
  id: number;
  skrot: string;
  nazwa: string;
  czy_domyslna: boolean;
  liczba_miejsc_po_przecinku: number;
}

export interface Waluta {
  id: number;
  skrot: string;
  nazwa: string;
  czy_domyslna: boolean;
}

export interface SposobPlatnosci {
  id: number;
  nazwa: string;
  liczba_dni: number;
  czy_domyslny: boolean;
  czy_zaplacone: boolean;
}

export interface Numerator {
  id: number;
  przeznaczenie: PrzeznaczenieNumeratora;
  format: string;
  grupa: string | null;
}

export interface Faktura {
  id: number;
  numer: string;
  data_wystawienia: string;
  data_sprzedazy: string;
  data_wprowadzenia: string;
  termin_platnosci: string;
  nip_sprzedawcy: string;
  nazwa_sprzedawcy: string;
  dane_sprzedawcy: string;
  nip_nabywcy: string;
  nazwa_nabywcy: string;
  dane_nabywcy: string;
  rachunek_bankowy: string;
  nazwa_banku: string;
  uwagi_publiczne: string;
  uwagi_wewnetrzne: string;
  razem_netto: string;
  razem_vat: string;
  razem_brutto: string;
  kurs_waluty: string;
  opis_sposobu_platnosci: string;
  rodzaj: RodzajFaktury;
  czy_wartosci_reczne: boolean;
  procedura_marzy: ProceduraMarzy;
  numer_ksef: string;
  sprzedawca_id: number | null;
  nabywca_id: number | null;
  faktura_korygowana_id: number | null;
  faktura_korygujaca_id: number | null;
  waluta_id: number | null;
  sposob_platnosci_id: number | null;
}

export const nowaFaktura = (rodzaj: RodzajFaktury): Faktura => {
  const dzis = new Date().toISOString().slice(0, 10);
  return {
    id: 0,
    numer: "",
    data_wystawienia: dzis,
    data_sprzedazy: dzis,
    data_wprowadzenia: dzis,
    termin_platnosci: dzis,
    nip_sprzedawcy: "",
    nazwa_sprzedawcy: "",
    dane_sprzedawcy: "",
    nip_nabywcy: "",
    nazwa_nabywcy: "",
    dane_nabywcy: "",
    rachunek_bankowy: "",
    nazwa_banku: "",
    uwagi_publiczne: "",
    uwagi_wewnetrzne: "",
    razem_netto: "0",
    razem_vat: "0",
    razem_brutto: "0",
    kurs_waluty: "0",
    opis_sposobu_platnosci: "",
    rodzaj,
    czy_wartosci_reczne: false,
    procedura_marzy: "NieDotyczy",
    numer_ksef: "",
    sprzedawca_id: null,
    nabywca_id: null,
    faktura_korygowana_id: null,
    faktura_korygujaca_id: null,
    waluta_id: null,
    sposob_platnosci_id: null,
  };
};

export interface PozycjaFaktury {
  id: number;
  faktura_id: number;
  towar_id: number | null;
  opis: string;
  cena_netto: string;
  cena_vat: string;
  cena_brutto: string;
  ilosc: string;
  wartosc_netto: string;
  wartosc_vat: string;
  wartosc_brutto: string;
  czy_wedlug_cen_brutto: boolean;
  czy_wartosci_reczne: boolean;
  stawka_vat_id: number | null;
  jednostka_miary_id: number | null;
  lp: number;
  czy_przed_korekta: boolean;
  gtu: number;
  stawka_ryczaltu: string | null;
  rabat_procent: string;
  rabat_cena: string;
  rabat_wartosc: string;
  cena_zakupu_dla_marzy: string;
}

export const nowaPozycja = (stawkaVatId: number | null, jednostkaId: number | null): PozycjaFaktury => ({
  id: 0,
  faktura_id: 0,
  towar_id: null,
  opis: "",
  cena_netto: "0",
  cena_vat: "0",
  cena_brutto: "0",
  ilosc: "1",
  wartosc_netto: "0",
  wartosc_vat: "0",
  wartosc_brutto: "0",
  czy_wedlug_cen_brutto: false,
  czy_wartosci_reczne: false,
  stawka_vat_id: stawkaVatId,
  jednostka_miary_id: jednostkaId,
  lp: 0,
  czy_przed_korekta: false,
  gtu: 0,
  stawka_ryczaltu: null,
  rabat_procent: "0",
  rabat_cena: "0",
  rabat_wartosc: "0",
  cena_zakupu_dla_marzy: "0",
});

export interface Wplata {
  id: number;
  faktura_id: number;
  data: string;
  kwota: string;
  uwagi: string;
  czy_rozliczenie: boolean;
}

export type FakturaListaWiersz = Faktura & {
  suma_wplat: string;
  pozostalo_do_zaplaty: string;
  waluta_skrot: string;
};

export interface FakturaSzczegoly {
  faktura: Faktura;
  pozycje: PozycjaFaktury[];
  wplaty: Wplata[];
}

export interface FakturaPrzeliczona {
  faktura: Faktura;
  pozycje: PozycjaFaktury[];
}

// ---------- komendy ----------

export const api = {
  listaKontrahentow: () => invoke<Kontrahent[]>("lista_kontrahentow"),
  zapiszKontrahenta: (kontrahent: Kontrahent) => invoke<number>("zapisz_kontrahenta", { kontrahent }),
  usunKontrahenta: (id: number) => invoke<void>("usun_kontrahenta", { id }),

  listaTowarow: () => invoke<Towar[]>("lista_towarow"),
  zapiszTowar: (towar: Towar) => invoke<number>("zapisz_towar", { towar }),
  usunTowar: (id: number) => invoke<void>("usun_towar", { id }),

  listaStawekVat: () => invoke<StawkaVat[]>("lista_stawek_vat"),
  zapiszStawkeVat: (stawka: StawkaVat) => invoke<number>("zapisz_stawke_vat", { stawka }),
  usunStawkeVat: (id: number) => invoke<void>("usun_stawke_vat", { id }),

  listaJednostek: () => invoke<JednostkaMiary[]>("lista_jednostek"),
  zapiszJednostke: (jednostka: JednostkaMiary) => invoke<number>("zapisz_jednostke", { jednostka }),
  usunJednostke: (id: number) => invoke<void>("usun_jednostke", { id }),

  listaWalut: () => invoke<Waluta[]>("lista_walut"),
  zapiszWalute: (waluta: Waluta) => invoke<number>("zapisz_walute", { waluta }),
  usunWalute: (id: number) => invoke<void>("usun_walute", { id }),

  listaSposobowPlatnosci: () => invoke<SposobPlatnosci[]>("lista_sposobow_platnosci"),
  zapiszSposobPlatnosci: (sposob: SposobPlatnosci) => invoke<number>("zapisz_sposob_platnosci", { sposob }),
  usunSposobPlatnosci: (id: number) => invoke<void>("usun_sposob_platnosci", { id }),

  listaNumeratorow: () => invoke<Numerator[]>("lista_numeratorow"),
  zapiszNumerator: (numerator: Numerator) => invoke<number>("zapisz_numerator", { numerator }),
  usunNumerator: (id: number) => invoke<void>("usun_numerator", { id }),

  listaFaktur: (czySprzedaz: boolean | null) =>
    invoke<FakturaListaWiersz[]>("lista_faktur", { czySprzedaz }),
  pobierzFakture: (id: number) => invoke<FakturaSzczegoly>("pobierz_fakture", { id }),
  przeliczFakture: (faktura: Faktura, pozycje: PozycjaFaktury[]) =>
    invoke<FakturaPrzeliczona>("przelicz_fakture", { faktura, pozycje }),
  zapiszFakture: (faktura: Faktura, pozycje: PozycjaFaktury[]) =>
    invoke<number>("zapisz_fakture", { faktura, pozycje }),
  wystawFakture: (id: number) => invoke<Faktura>("wystaw_fakture", { id }),
  usunFakture: (id: number) => invoke<void>("usun_fakture", { id }),
  przygotujKorekte: (id: number) => invoke<number>("przygotuj_korekte", { id }),
  przygotujPodobna: (id: number) => invoke<number>("przygotuj_podobna", { id }),
  dodajWplate: (wplata: Wplata) => invoke<number>("dodaj_wplate", { wplata }),
  usunWplate: (id: number) => invoke<void>("usun_wplate", { id }),
};

export const kwota = (s: string): string => {
  const n = Number(s);
  return Number.isFinite(n)
    ? n.toLocaleString("pl-PL", { minimumFractionDigits: 2, maximumFractionDigits: 2 })
    : s;
};
