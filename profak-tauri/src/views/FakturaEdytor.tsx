import { useEffect, useRef, useState } from "react";
import {
  api,
  Faktura,
  JednostkaMiary,
  Kontrahent,
  kwota,
  nowaFaktura,
  nowaPozycja,
  PozycjaFaktury,
  RodzajFaktury,
  SposobPlatnosci,
  StawkaVat,
  Towar,
  Waluta,
  Wplata,
} from "../api";

interface Props {
  fakturaId: number | null;
  rodzaj: RodzajFaktury;
  zamknij: () => void;
}

export function FakturaEdytor({ fakturaId, rodzaj, zamknij }: Props) {
  const [faktura, setFaktura] = useState<Faktura>(() => nowaFaktura(rodzaj));
  const [pozycje, setPozycje] = useState<PozycjaFaktury[]>([]);
  const [wplaty, setWplaty] = useState<Wplata[]>([]);
  const [kontrahenci, setKontrahenci] = useState<Kontrahent[]>([]);
  const [towary, setTowary] = useState<Towar[]>([]);
  const [stawki, setStawki] = useState<StawkaVat[]>([]);
  const [jednostki, setJednostki] = useState<JednostkaMiary[]>([]);
  const [waluty, setWaluty] = useState<Waluta[]>([]);
  const [sposoby, setSposoby] = useState<SposobPlatnosci[]>([]);
  const [blad, setBlad] = useState("");
  const [nowaWplata, setNowaWplata] = useState({ data: new Date().toISOString().slice(0, 10), kwota: "" });
  const licznikPrzeliczen = useRef(0);

  const czySprzedaz = !["Zakup", "KorektaZakupu", "DowódWewnętrzny", "Usunięta"].includes(faktura.rodzaj);

  useEffect(() => {
    Promise.all([
      api.listaKontrahentow(),
      api.listaTowarow(),
      api.listaStawekVat(),
      api.listaJednostek(),
      api.listaWalut(),
      api.listaSposobowPlatnosci(),
    ])
      .then(([k, t, s, j, w, sp]) => {
        setKontrahenci(k);
        setTowary(t);
        setStawki(s);
        setJednostki(j);
        setWaluty(w);
        setSposoby(sp);
        if (fakturaId === null) {
          const podmiot = k.find((x) => x.czy_podmiot);
          const walutaDomyslna = w.find((x) => x.czy_domyslna);
          const sposobDomyslny = sp.find((x) => x.czy_domyslny);
          setFaktura((f) => {
            const nowa = { ...f };
            nowa.waluta_id = walutaDomyslna?.id ?? null;
            if (sposobDomyslny) {
              nowa.sposob_platnosci_id = sposobDomyslny.id;
              nowa.opis_sposobu_platnosci = sposobDomyslny.nazwa;
              nowa.termin_platnosci = dodajDni(nowa.data_wystawienia, sposobDomyslny.liczba_dni);
            }
            if (podmiot && czySprzedaz) ustawKontrahenta(nowa, podmiot, "sprzedawca");
            return nowa;
          });
        }
      })
      .catch((e) => setBlad(String(e)));

    if (fakturaId !== null) {
      api
        .pobierzFakture(fakturaId)
        .then((szczegoly) => {
          setFaktura(szczegoly.faktura);
          setPozycje(szczegoly.pozycje);
          setWplaty(szczegoly.wplaty);
        })
        .catch((e) => setBlad(String(e)));
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fakturaId]);

  const przelicz = (f: Faktura, p: PozycjaFaktury[]) => {
    const numer = ++licznikPrzeliczen.current;
    api
      .przeliczFakture(f, p)
      .then((wynik) => {
        if (numer !== licznikPrzeliczen.current) return; // odpowiedź nieaktualna
        setFaktura(wynik.faktura);
        setPozycje(wynik.pozycje);
      })
      .catch((e) => setBlad(String(e)));
  };

  const zmienFakture = (zmiany: Partial<Faktura>) => {
    const nowa = { ...faktura, ...zmiany };
    setFaktura(nowa);
    return nowa;
  };

  const zmienPozycje = (indeks: number, zmiany: Partial<PozycjaFaktury>) => {
    const nowe = pozycje.map((p, i) => (i === indeks ? { ...p, ...zmiany } : p));
    setPozycje(nowe);
    przelicz(faktura, nowe);
  };

  const dodajPozycje = () => {
    const stawkaDomyslna = stawki.find((s) => s.czy_domyslna);
    const jednostkaDomyslna = jednostki.find((j) => j.czy_domyslna);
    const nowe = [...pozycje, nowaPozycja(stawkaDomyslna?.id ?? null, jednostkaDomyslna?.id ?? null)];
    setPozycje(nowe);
    przelicz(faktura, nowe);
  };

  const usunPozycje = (indeks: number) => {
    const nowe = pozycje.filter((_, i) => i !== indeks);
    setPozycje(nowe);
    przelicz(faktura, nowe);
  };

  const wybierzTowar = (indeks: number, towarId: number | null) => {
    const towar = towary.find((t) => t.id === towarId);
    if (!towar) {
      zmienPozycje(indeks, { towar_id: null });
      return;
    }
    // Odpowiednik PozycjaFaktury.UstawTowar.
    zmienPozycje(indeks, {
      towar_id: towar.id,
      opis: towar.nazwa,
      jednostka_miary_id: towar.jednostka_miary_id,
      stawka_vat_id: towar.stawka_vat_id,
      gtu: towar.gtu,
      stawka_ryczaltu: towar.stawka_ryczaltu,
      czy_wedlug_cen_brutto: towar.sposob_liczenia_ceny === "WedługBrutto",
      cena_netto: towar.cena_netto,
      cena_brutto: towar.cena_brutto,
    });
  };

  const ustawKontrahenta = (f: Faktura, k: Kontrahent, rola: "sprzedawca" | "nabywca") => {
    if (rola === "sprzedawca") {
      f.sprzedawca_id = k.id;
      f.nip_sprzedawcy = k.nip;
      f.nazwa_sprzedawcy = k.pelna_nazwa || k.nazwa;
      f.dane_sprzedawcy = k.adres_rejestrowy;
      f.rachunek_bankowy = k.rachunek_bankowy;
      f.nazwa_banku = k.nazwa_banku;
    } else {
      f.nabywca_id = k.id;
      f.nip_nabywcy = k.nip;
      f.nazwa_nabywcy = k.pelna_nazwa || k.nazwa;
      f.dane_nabywcy = k.adres_rejestrowy;
    }
  };

  const wybierzKontrahenta = (rola: "sprzedawca" | "nabywca", id: number | null) => {
    const k = kontrahenci.find((x) => x.id === id);
    const nowa = { ...faktura };
    if (k) ustawKontrahenta(nowa, k, rola);
    else if (rola === "sprzedawca") nowa.sprzedawca_id = null;
    else nowa.nabywca_id = null;
    setFaktura(nowa);
  };

  const wybierzSposobPlatnosci = (id: number | null) => {
    const sposob = sposoby.find((s) => s.id === id);
    const nowa = { ...faktura, sposob_platnosci_id: id };
    if (sposob) {
      nowa.opis_sposobu_platnosci = sposob.nazwa;
      nowa.termin_platnosci = dodajDni(nowa.data_wystawienia, sposob.liczba_dni);
    }
    setFaktura(nowa);
  };

  const zapisz = async (wystaw: boolean) => {
    try {
      const id = await api.zapiszFakture(faktura, pozycje);
      if (wystaw) await api.wystawFakture(id);
      zamknij();
    } catch (e) {
      setBlad(String(e));
    }
  };

  const dodajWplate = async () => {
    if (faktura.id === 0) return;
    try {
      await api.dodajWplate({
        id: 0,
        faktura_id: faktura.id,
        data: nowaWplata.data,
        kwota: nowaWplata.kwota || faktura.razem_brutto,
        uwagi: "",
        czy_rozliczenie: false,
      });
      const szczegoly = await api.pobierzFakture(faktura.id);
      setWplaty(szczegoly.wplaty);
      setNowaWplata({ data: new Date().toISOString().slice(0, 10), kwota: "" });
    } catch (e) {
      setBlad(String(e));
    }
  };

  const usunWplate = async (id: number) => {
    try {
      await api.usunWplate(id);
      setWplaty(wplaty.filter((w) => w.id !== id));
    } catch (e) {
      setBlad(String(e));
    }
  };

  const sumaWplat = wplaty.reduce((suma, w) => suma + Number(w.kwota), 0);

  return (
    <section>
      <header className="naglowek-widoku">
        <h2>
          {faktura.rodzaj} {faktura.numer && <span className="numer-faktury">{faktura.numer}</span>}
        </h2>
        {faktura.id !== 0 && (
          <button
            onClick={() =>
              api.wydrukujFakture(faktura.id).then(
                () => setBlad(""),
                (e) => setBlad(String(e)),
              )
            }
          >
            PDF
          </button>
        )}
        <button onClick={() => zapisz(false)}>Zapisz roboczą</button>
        <button className="glowny" onClick={() => zapisz(true)}>
          {faktura.numer ? "Zapisz" : "Wystaw"}
        </button>
        <button onClick={zamknij}>Anuluj</button>
      </header>
      {blad && <p className="blad">{blad}</p>}

      <div className="siatka-formularza">
        <fieldset>
          <legend>Daty i płatność</legend>
          <label>
            Data wystawienia
            <input
              type="date"
              value={faktura.data_wystawienia}
              onChange={(e) => zmienFakture({ data_wystawienia: e.target.value })}
            />
          </label>
          <label>
            Data sprzedaży
            <input
              type="date"
              value={faktura.data_sprzedazy}
              onChange={(e) => zmienFakture({ data_sprzedazy: e.target.value })}
            />
          </label>
          <label>
            Sposób płatności
            <select
              value={faktura.sposob_platnosci_id ?? ""}
              onChange={(e) => wybierzSposobPlatnosci(e.target.value ? Number(e.target.value) : null)}
            >
              <option value="">—</option>
              {sposoby.map((s) => (
                <option key={s.id} value={s.id}>
                  {s.nazwa}
                </option>
              ))}
            </select>
          </label>
          <label>
            Termin płatności
            <input
              type="date"
              value={faktura.termin_platnosci}
              onChange={(e) => zmienFakture({ termin_platnosci: e.target.value })}
            />
          </label>
          <label>
            Waluta
            <select
              value={faktura.waluta_id ?? ""}
              onChange={(e) => zmienFakture({ waluta_id: e.target.value ? Number(e.target.value) : null })}
            >
              <option value="">—</option>
              {waluty.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.skrot}
                </option>
              ))}
            </select>
          </label>
        </fieldset>

        <fieldset>
          <legend>Sprzedawca</legend>
          <label>
            Kontrahent
            <select
              value={faktura.sprzedawca_id ?? ""}
              onChange={(e) => wybierzKontrahenta("sprzedawca", e.target.value ? Number(e.target.value) : null)}
            >
              <option value="">—</option>
              {kontrahenci.map((k) => (
                <option key={k.id} value={k.id}>
                  {k.nazwa}
                </option>
              ))}
            </select>
          </label>
          <label>
            Nazwa
            <input value={faktura.nazwa_sprzedawcy} onChange={(e) => zmienFakture({ nazwa_sprzedawcy: e.target.value })} />
          </label>
          <label>
            NIP
            <input value={faktura.nip_sprzedawcy} onChange={(e) => zmienFakture({ nip_sprzedawcy: e.target.value })} />
          </label>
          <label>
            Adres
            <textarea value={faktura.dane_sprzedawcy} onChange={(e) => zmienFakture({ dane_sprzedawcy: e.target.value })} />
          </label>
        </fieldset>

        <fieldset>
          <legend>Nabywca</legend>
          <label>
            Kontrahent
            <select
              value={faktura.nabywca_id ?? ""}
              onChange={(e) => wybierzKontrahenta("nabywca", e.target.value ? Number(e.target.value) : null)}
            >
              <option value="">—</option>
              {kontrahenci.map((k) => (
                <option key={k.id} value={k.id}>
                  {k.nazwa}
                </option>
              ))}
            </select>
          </label>
          <label>
            Nazwa
            <input value={faktura.nazwa_nabywcy} onChange={(e) => zmienFakture({ nazwa_nabywcy: e.target.value })} />
          </label>
          <label>
            NIP
            <input value={faktura.nip_nabywcy} onChange={(e) => zmienFakture({ nip_nabywcy: e.target.value })} />
          </label>
          <label>
            Adres
            <textarea value={faktura.dane_nabywcy} onChange={(e) => zmienFakture({ dane_nabywcy: e.target.value })} />
          </label>
        </fieldset>
      </div>

      <h3>Pozycje</h3>
      <table className="tabela pozycje">
        <thead>
          <tr>
            <th>LP</th>
            <th>Towar/usługa</th>
            <th>Opis</th>
            <th className="num">Ilość</th>
            <th>JM</th>
            <th className="num">Cena netto</th>
            <th className="num">Cena brutto</th>
            <th>VAT</th>
            <th className="num">Rabat %</th>
            <th className="num">Wartość netto</th>
            <th className="num">Wartość VAT</th>
            <th className="num">Wartość brutto</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {pozycje.map((p, i) => (
            <tr key={i} className={p.czy_przed_korekta ? "przed-korekta" : ""}>
              <td>{p.czy_przed_korekta ? "przed" : p.lp}</td>
              <td>
                <select value={p.towar_id ?? ""} onChange={(e) => wybierzTowar(i, e.target.value ? Number(e.target.value) : null)}>
                  <option value="">—</option>
                  {towary.map((t) => (
                    <option key={t.id} value={t.id}>
                      {t.nazwa}
                    </option>
                  ))}
                </select>
              </td>
              <td>
                <input value={p.opis} onChange={(e) => zmienPozycje(i, { opis: e.target.value })} />
              </td>
              <td className="num">
                <input className="num" value={p.ilosc} onChange={(e) => zmienPozycje(i, { ilosc: e.target.value || "0" })} />
              </td>
              <td>
                <select
                  value={p.jednostka_miary_id ?? ""}
                  onChange={(e) => zmienPozycje(i, { jednostka_miary_id: e.target.value ? Number(e.target.value) : null })}
                >
                  <option value="">—</option>
                  {jednostki.map((j) => (
                    <option key={j.id} value={j.id}>
                      {j.skrot}
                    </option>
                  ))}
                </select>
              </td>
              <td className="num">
                <input
                  className="num"
                  disabled={p.czy_wedlug_cen_brutto}
                  value={p.cena_netto}
                  onChange={(e) => zmienPozycje(i, { cena_netto: e.target.value || "0", czy_wedlug_cen_brutto: false })}
                />
              </td>
              <td className="num">
                <input
                  className="num"
                  value={p.cena_brutto}
                  onChange={(e) => zmienPozycje(i, { cena_brutto: e.target.value || "0", czy_wedlug_cen_brutto: true })}
                />
              </td>
              <td>
                <select
                  value={p.stawka_vat_id ?? ""}
                  onChange={(e) => zmienPozycje(i, { stawka_vat_id: e.target.value ? Number(e.target.value) : null })}
                >
                  <option value="">—</option>
                  {stawki.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.skrot}
                    </option>
                  ))}
                </select>
              </td>
              <td className="num">
                <input className="num" value={p.rabat_procent} onChange={(e) => zmienPozycje(i, { rabat_procent: e.target.value || "0" })} />
              </td>
              <td className="num">{kwota(p.wartosc_netto)}</td>
              <td className="num">{kwota(p.wartosc_vat)}</td>
              <td className="num">{kwota(p.wartosc_brutto)}</td>
              <td>
                <button className="usun" onClick={() => usunPozycje(i)}>
                  ✕
                </button>
              </td>
            </tr>
          ))}
        </tbody>
        <tfoot>
          <tr>
            <td colSpan={9}>
              <button onClick={dodajPozycje}>+ Dodaj pozycję</button>
            </td>
            <td className="num razem">{kwota(faktura.razem_netto)}</td>
            <td className="num razem">{kwota(faktura.razem_vat)}</td>
            <td className="num razem">{kwota(faktura.razem_brutto)}</td>
            <td></td>
          </tr>
        </tfoot>
      </table>

      <div className="siatka-formularza">
        <fieldset>
          <legend>Uwagi</legend>
          <label>
            Uwagi na fakturze
            <textarea value={faktura.uwagi_publiczne} onChange={(e) => zmienFakture({ uwagi_publiczne: e.target.value })} />
          </label>
          <label>
            Uwagi wewnętrzne
            <textarea value={faktura.uwagi_wewnetrzne} onChange={(e) => zmienFakture({ uwagi_wewnetrzne: e.target.value })} />
          </label>
        </fieldset>

        {faktura.id !== 0 && (
          <fieldset>
            <legend>
              Wpłaty (razem {sumaWplat.toLocaleString("pl-PL", { minimumFractionDigits: 2 })} / {kwota(faktura.razem_brutto)})
            </legend>
            <table className="tabela">
              <tbody>
                {wplaty.map((w) => (
                  <tr key={w.id}>
                    <td>{w.data}</td>
                    <td className="num">{kwota(w.kwota)}</td>
                    <td>
                      <button className="usun" onClick={() => usunWplate(w.id)}>
                        ✕
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <div className="wiersz">
              <input type="date" value={nowaWplata.data} onChange={(e) => setNowaWplata({ ...nowaWplata, data: e.target.value })} />
              <input
                className="num"
                placeholder={`kwota (domyślnie ${kwota(faktura.razem_brutto)})`}
                value={nowaWplata.kwota}
                onChange={(e) => setNowaWplata({ ...nowaWplata, kwota: e.target.value })}
              />
              <button onClick={dodajWplate}>Dodaj wpłatę</button>
            </div>
          </fieldset>
        )}
      </div>
    </section>
  );
}

function dodajDni(dataISO: string, dni: number): string {
  const d = new Date(dataISO + "T00:00:00");
  d.setDate(d.getDate() + dni);
  return d.toISOString().slice(0, 10);
}
