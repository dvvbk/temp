//! Powłoka Tauri - cienka warstwa komend nad profak-core.

use std::sync::Mutex;

use profak_core::model::*;
use profak_core::rusqlite::Connection;
use profak_core::{db, repo};
use tauri::{Manager, State};

pub struct Baza(pub Mutex<Connection>);

type Wynik<T> = Result<T, String>;

fn mapuj<T>(wynik: profak_core::Wynik<T>) -> Wynik<T> {
    wynik.map_err(|e| e.to_string())
}

// ---------- Kontrahenci ----------

#[tauri::command]
fn lista_kontrahentow(baza: State<Baza>) -> Wynik<Vec<Kontrahent>> {
    mapuj(repo::lista_kontrahentow(&baza.0.lock().unwrap()))
}

#[tauri::command]
fn zapisz_kontrahenta(baza: State<Baza>, kontrahent: Kontrahent) -> Wynik<i64> {
    mapuj(repo::zapisz_kontrahenta(&baza.0.lock().unwrap(), &kontrahent))
}

#[tauri::command]
fn usun_kontrahenta(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_kontrahenta(&baza.0.lock().unwrap(), id))
}

// ---------- Towary ----------

#[tauri::command]
fn lista_towarow(baza: State<Baza>) -> Wynik<Vec<Towar>> {
    mapuj(repo::lista_towarow(&baza.0.lock().unwrap()))
}

#[tauri::command]
fn zapisz_towar(baza: State<Baza>, towar: Towar) -> Wynik<i64> {
    mapuj(repo::zapisz_towar(&baza.0.lock().unwrap(), &towar))
}

#[tauri::command]
fn usun_towar(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_towar(&baza.0.lock().unwrap(), id))
}

// ---------- Słowniki ----------

#[tauri::command]
fn lista_stawek_vat(baza: State<Baza>) -> Wynik<Vec<StawkaVat>> {
    mapuj(repo::lista_stawek_vat(&baza.0.lock().unwrap()))
}

#[tauri::command]
fn zapisz_stawke_vat(baza: State<Baza>, stawka: StawkaVat) -> Wynik<i64> {
    mapuj(repo::zapisz_stawke_vat(&baza.0.lock().unwrap(), &stawka))
}

#[tauri::command]
fn usun_stawke_vat(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_stawke_vat(&baza.0.lock().unwrap(), id))
}

#[tauri::command]
fn lista_jednostek(baza: State<Baza>) -> Wynik<Vec<JednostkaMiary>> {
    mapuj(repo::lista_jednostek(&baza.0.lock().unwrap()))
}

#[tauri::command]
fn zapisz_jednostke(baza: State<Baza>, jednostka: JednostkaMiary) -> Wynik<i64> {
    mapuj(repo::zapisz_jednostke(&baza.0.lock().unwrap(), &jednostka))
}

#[tauri::command]
fn usun_jednostke(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_jednostke(&baza.0.lock().unwrap(), id))
}

#[tauri::command]
fn lista_walut(baza: State<Baza>) -> Wynik<Vec<Waluta>> {
    mapuj(repo::lista_walut(&baza.0.lock().unwrap()))
}

#[tauri::command]
fn zapisz_walute(baza: State<Baza>, waluta: Waluta) -> Wynik<i64> {
    mapuj(repo::zapisz_walute(&baza.0.lock().unwrap(), &waluta))
}

#[tauri::command]
fn usun_walute(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_walute(&baza.0.lock().unwrap(), id))
}

#[tauri::command]
fn lista_sposobow_platnosci(baza: State<Baza>) -> Wynik<Vec<SposobPlatnosci>> {
    mapuj(repo::lista_sposobow_platnosci(&baza.0.lock().unwrap()))
}

#[tauri::command]
fn zapisz_sposob_platnosci(baza: State<Baza>, sposob: SposobPlatnosci) -> Wynik<i64> {
    mapuj(repo::zapisz_sposob_platnosci(&baza.0.lock().unwrap(), &sposob))
}

#[tauri::command]
fn usun_sposob_platnosci(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_sposob_platnosci(&baza.0.lock().unwrap(), id))
}

#[tauri::command]
fn lista_numeratorow(baza: State<Baza>) -> Wynik<Vec<Numerator>> {
    mapuj(repo::lista_numeratorow(&baza.0.lock().unwrap()))
}

#[tauri::command]
fn zapisz_numerator(baza: State<Baza>, numerator: Numerator) -> Wynik<i64> {
    mapuj(repo::zapisz_numerator(&baza.0.lock().unwrap(), &numerator))
}

#[tauri::command]
fn usun_numerator(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_numerator(&baza.0.lock().unwrap(), id))
}

// ---------- Faktury ----------

#[tauri::command]
fn lista_faktur(baza: State<Baza>, czy_sprzedaz: Option<bool>) -> Wynik<Vec<FakturaListaWiersz>> {
    mapuj(repo::lista_faktur(&baza.0.lock().unwrap(), czy_sprzedaz))
}

#[derive(serde::Serialize)]
pub struct FakturaSzczegoly {
    pub faktura: Faktura,
    pub pozycje: Vec<PozycjaFaktury>,
    pub wplaty: Vec<Wplata>,
}

#[tauri::command]
fn pobierz_fakture(baza: State<Baza>, id: i64) -> Wynik<FakturaSzczegoly> {
    let conn = baza.0.lock().unwrap();
    let faktura = mapuj(repo::pobierz_fakture(&conn, id))?
        .ok_or_else(|| "Nie znaleziono faktury.".to_string())?;
    let pozycje = mapuj(repo::pozycje_faktury(&conn, id))?;
    let wplaty = mapuj(repo::wplaty_faktury(&conn, id))?;
    Ok(FakturaSzczegoly { faktura, pozycje, wplaty })
}

#[derive(serde::Serialize)]
pub struct FakturaPrzeliczona {
    pub faktura: Faktura,
    pub pozycje: Vec<PozycjaFaktury>,
}

#[tauri::command]
fn przelicz_fakture(
    baza: State<Baza>,
    faktura: Faktura,
    pozycje: Vec<PozycjaFaktury>,
) -> Wynik<FakturaPrzeliczona> {
    let conn = baza.0.lock().unwrap();
    let (faktura, pozycje) = mapuj(repo::przelicz_fakture(&conn, faktura, pozycje))?;
    Ok(FakturaPrzeliczona { faktura, pozycje })
}

#[tauri::command]
fn zapisz_fakture(baza: State<Baza>, faktura: Faktura, pozycje: Vec<PozycjaFaktury>) -> Wynik<i64> {
    mapuj(repo::zapisz_fakture(&mut baza.0.lock().unwrap(), faktura, pozycje))
}

#[tauri::command]
fn wystaw_fakture(baza: State<Baza>, id: i64) -> Wynik<Faktura> {
    mapuj(repo::wystaw_fakture(&mut baza.0.lock().unwrap(), id))
}

#[tauri::command]
fn usun_fakture(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_fakture(&baza.0.lock().unwrap(), id))
}

#[tauri::command]
fn przygotuj_korekte(baza: State<Baza>, id: i64) -> Wynik<i64> {
    mapuj(repo::przygotuj_korekte(&mut baza.0.lock().unwrap(), id))
}

#[tauri::command]
fn przygotuj_podobna(baza: State<Baza>, id: i64) -> Wynik<i64> {
    mapuj(repo::przygotuj_podobna(&mut baza.0.lock().unwrap(), id))
}

#[tauri::command]
fn dodaj_wplate(baza: State<Baza>, wplata: Wplata) -> Wynik<i64> {
    mapuj(repo::dodaj_wplate(&baza.0.lock().unwrap(), &wplata))
}

#[tauri::command]
fn usun_wplate(baza: State<Baza>, id: i64) -> Wynik<()> {
    mapuj(repo::usun_wplate(&baza.0.lock().unwrap(), id))
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let katalog = app.path().app_data_dir()?;
            std::fs::create_dir_all(&katalog)?;
            let sciezka = katalog.join("profak.db");
            let conn = db::otworz(&sciezka).map_err(|e| std::io::Error::other(e.to_string()))?;
            app.manage(Baza(Mutex::new(conn)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            lista_kontrahentow,
            zapisz_kontrahenta,
            usun_kontrahenta,
            lista_towarow,
            zapisz_towar,
            usun_towar,
            lista_stawek_vat,
            zapisz_stawke_vat,
            usun_stawke_vat,
            lista_jednostek,
            zapisz_jednostke,
            usun_jednostke,
            lista_walut,
            zapisz_walute,
            usun_walute,
            lista_sposobow_platnosci,
            zapisz_sposob_platnosci,
            usun_sposob_platnosci,
            lista_numeratorow,
            zapisz_numerator,
            usun_numerator,
            lista_faktur,
            pobierz_fakture,
            przelicz_fakture,
            zapisz_fakture,
            wystaw_fakture,
            usun_fakture,
            przygotuj_korekte,
            przygotuj_podobna,
            dodaj_wplate,
            usun_wplate
        ])
        .run(tauri::generate_context!())
        .expect("błąd uruchamiania aplikacji ProFak");
}
