# ProFak (port na Tauri)

Port [ProFaka](https://github.com/lkosson/profak) — prostego, darmowego programu do
fakturowania — z C#/WinForms na **Tauri 2** (backend w Rust, frontend w React/TypeScript).

## Architektura

```
profak-tauri/
├── crates/profak-core/   # rdzeń logiki (czysty Rust, bez zależności od UI)
│   ├── model.rs          # encje: Faktura, PozycjaFaktury, Kontrahent, Towar, słowniki
│   ├── db.rs             # schemat SQLite + dane startowe (jak DaneStartowe.cs)
│   ├── faktura.rs        # wyliczenia: PrzeliczCeny, PrzeliczRazem (zgodne z oryginałem)
│   ├── numerator.rs      # numeracja dokumentów: FV/[Numer]/[Rok], grupy liczników
│   ├── liczby.rs         # zaokrąglanie kwot (MidpointRounding.AwayFromZero)
│   └── repo.rs           # CRUD + operacje: wystawianie, korekta, faktura podobna
├── src-tauri/            # powłoka Tauri - komendy IPC nad profak-core
└── src/                  # frontend React: listy faktur, edytor, kontrahenci, słowniki
```

Kwoty są reprezentowane jako `rust_decimal::Decimal` (odpowiednik `decimal` z C#)
i serializowane do frontendu jako stringi — bez utraty precyzji na liczbach
zmiennoprzecinkowych. Baza danych to plikowy SQLite w katalogu danych aplikacji.

## Co jest przeniesione

* Faktury sprzedaży (VAT, proforma, VAT marża, rachunek, dowód wewnętrzny) i zakupu
* Pozycje faktur z wyliczeniami zgodnymi 1:1 z oryginałem: ceny wg netto/brutto,
  VAT marża od ceny zakupu, rabaty (procentowy, kwotowy od ceny i od wartości),
  obcinanie ujemnych wartości, zaokrąglanie „od zera"
* Numeratory z formatami (`FV/[Numer]/[Rok]`, `[Numer:000]`, `[Miesiąc]`, …)
  i osobnymi licznikami per grupa (np. reset numeracji co rok)
* Wystawianie: nadanie numeru + automatyczna wpłata przy płatności gotówką/kartą
* Korekty (pozycje „przed/po korekcie") i wystawianie faktur podobnych
* Wpłaty i kwota pozostała do zapłaty
* Kontrahenci, towary/usługi, stawki VAT, jednostki miar, waluty, sposoby płatności
* Dane startowe identyczne z oryginałem (stawki VAT, numeratory, sposoby płatności…)

## Czego (jeszcze) nie ma

Wydruki (QuestPDF), JPK/KSeF, integracje GUS i biała lista, deklaracje VAT,
zaliczki PIT, składki ZUS, KPiR/EP, wysyłka e-mail, załączniki, API zewnętrzne.
Rdzeń w `profak-core` jest tak podzielony, żeby te moduły dało się dokładać
bez zmian w UI.

## Uruchomienie (dev)

Wymagania: Rust (stable), Node.js 20+, a na Linuksie pakiety
`libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev`.

```sh
npm install
npm run tauri dev
```

## Budowanie paczki

```sh
npm run tauri build
```

## Testy rdzenia

Logika domenowa (wyliczenia, numeracja, korekty, wpłaty) jest pokryta testami:

```sh
cargo test --manifest-path crates/profak-core/Cargo.toml
```
