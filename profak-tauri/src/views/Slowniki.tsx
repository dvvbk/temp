import { useCallback, useEffect, useState } from "react";
import { api, JednostkaMiary, Numerator, PrzeznaczenieNumeratora, SposobPlatnosci, StawkaVat, Waluta } from "../api";

type Zakladka = "vat" | "jednostki" | "platnosci" | "waluty" | "numeracja";

const zakladki: { id: Zakladka; etykieta: string }[] = [
  { id: "vat", etykieta: "Stawki VAT" },
  { id: "jednostki", etykieta: "Jednostki miar" },
  { id: "platnosci", etykieta: "Sposoby płatności" },
  { id: "waluty", etykieta: "Waluty" },
  { id: "numeracja", etykieta: "Numeracja" },
];

const przeznaczenia: PrzeznaczenieNumeratora[] = [
  "Faktura",
  "Proforma",
  "KorektaSprzedaży",
  "DowódWewnętrzny",
  "VatMarża",
  "KorektaVatMarży",
  "Rachunek",
  "KorektaRachunku",
];

export function Slowniki() {
  const [zakladka, setZakladka] = useState<Zakladka>("vat");
  const [stawki, setStawki] = useState<StawkaVat[]>([]);
  const [jednostki, setJednostki] = useState<JednostkaMiary[]>([]);
  const [sposoby, setSposoby] = useState<SposobPlatnosci[]>([]);
  const [waluty, setWaluty] = useState<Waluta[]>([]);
  const [numeratory, setNumeratory] = useState<Numerator[]>([]);
  const [blad, setBlad] = useState("");

  const odswiez = useCallback(() => {
    Promise.all([
      api.listaStawekVat(),
      api.listaJednostek(),
      api.listaSposobowPlatnosci(),
      api.listaWalut(),
      api.listaNumeratorow(),
    ])
      .then(([s, j, sp, w, n]) => {
        setStawki(s);
        setJednostki(j);
        setSposoby(sp);
        setWaluty(w);
        setNumeratory(n);
      })
      .catch((e) => setBlad(String(e)));
  }, []);

  useEffect(odswiez, [odswiez]);

  const wykonaj = async (akcja: () => Promise<unknown>) => {
    try {
      await akcja();
      odswiez();
    } catch (e) {
      setBlad(String(e));
    }
  };

  return (
    <section>
      <header className="naglowek-widoku">
        <h2>Słowniki</h2>
        {zakladki.map((z) => (
          <button key={z.id} className={zakladka === z.id ? "glowny" : ""} onClick={() => setZakladka(z.id)}>
            {z.etykieta}
          </button>
        ))}
      </header>
      {blad && <p className="blad">{blad}</p>}

      {zakladka === "vat" && (
        <Slownik<StawkaVat>
          wiersze={stawki}
          kolumny={[
            { naglowek: "Skrót", pole: (s, zmien) => <input value={s.skrot} onChange={(e) => zmien({ ...s, skrot: e.target.value })} /> },
            {
              naglowek: "Wartość %",
              pole: (s, zmien) => <input className="num" value={s.wartosc} onChange={(e) => zmien({ ...s, wartosc: e.target.value || "0" })} />,
            },
            {
              naglowek: "Domyślna",
              pole: (s, zmien) => (
                <input type="checkbox" checked={s.czy_domyslna} onChange={(e) => zmien({ ...s, czy_domyslna: e.target.checked })} />
              ),
            },
          ]}
          nowy={() => ({ id: 0, skrot: "", wartosc: "0", czy_domyslna: false })}
          zapisz={(s) => wykonaj(() => api.zapiszStawkeVat(s))}
          usun={(s) => wykonaj(() => api.usunStawkeVat(s.id))}
        />
      )}

      {zakladka === "jednostki" && (
        <Slownik<JednostkaMiary>
          wiersze={jednostki}
          kolumny={[
            { naglowek: "Skrót", pole: (j, zmien) => <input value={j.skrot} onChange={(e) => zmien({ ...j, skrot: e.target.value })} /> },
            { naglowek: "Nazwa", pole: (j, zmien) => <input value={j.nazwa} onChange={(e) => zmien({ ...j, nazwa: e.target.value })} /> },
            {
              naglowek: "Miejsca po przecinku",
              pole: (j, zmien) => (
                <input
                  className="num"
                  value={j.liczba_miejsc_po_przecinku}
                  onChange={(e) => zmien({ ...j, liczba_miejsc_po_przecinku: Number(e.target.value) || 0 })}
                />
              ),
            },
            {
              naglowek: "Domyślna",
              pole: (j, zmien) => (
                <input type="checkbox" checked={j.czy_domyslna} onChange={(e) => zmien({ ...j, czy_domyslna: e.target.checked })} />
              ),
            },
          ]}
          nowy={() => ({ id: 0, skrot: "", nazwa: "", czy_domyslna: false, liczba_miejsc_po_przecinku: 0 })}
          zapisz={(j) => wykonaj(() => api.zapiszJednostke(j))}
          usun={(j) => wykonaj(() => api.usunJednostke(j.id))}
        />
      )}

      {zakladka === "platnosci" && (
        <Slownik<SposobPlatnosci>
          wiersze={sposoby}
          kolumny={[
            { naglowek: "Nazwa", pole: (s, zmien) => <input value={s.nazwa} onChange={(e) => zmien({ ...s, nazwa: e.target.value })} /> },
            {
              naglowek: "Liczba dni",
              pole: (s, zmien) => (
                <input className="num" value={s.liczba_dni} onChange={(e) => zmien({ ...s, liczba_dni: Number(e.target.value) || 0 })} />
              ),
            },
            {
              naglowek: "Domyślny",
              pole: (s, zmien) => (
                <input type="checkbox" checked={s.czy_domyslny} onChange={(e) => zmien({ ...s, czy_domyslny: e.target.checked })} />
              ),
            },
            {
              naglowek: "Od razu zapłacone",
              pole: (s, zmien) => (
                <input type="checkbox" checked={s.czy_zaplacone} onChange={(e) => zmien({ ...s, czy_zaplacone: e.target.checked })} />
              ),
            },
          ]}
          nowy={() => ({ id: 0, nazwa: "", liczba_dni: 0, czy_domyslny: false, czy_zaplacone: false })}
          zapisz={(s) => wykonaj(() => api.zapiszSposobPlatnosci(s))}
          usun={(s) => wykonaj(() => api.usunSposobPlatnosci(s.id))}
        />
      )}

      {zakladka === "waluty" && (
        <Slownik<Waluta>
          wiersze={waluty}
          kolumny={[
            { naglowek: "Skrót", pole: (w, zmien) => <input value={w.skrot} onChange={(e) => zmien({ ...w, skrot: e.target.value })} /> },
            { naglowek: "Nazwa", pole: (w, zmien) => <input value={w.nazwa} onChange={(e) => zmien({ ...w, nazwa: e.target.value })} /> },
            {
              naglowek: "Domyślna",
              pole: (w, zmien) => (
                <input type="checkbox" checked={w.czy_domyslna} onChange={(e) => zmien({ ...w, czy_domyslna: e.target.checked })} />
              ),
            },
          ]}
          nowy={() => ({ id: 0, skrot: "", nazwa: "", czy_domyslna: false })}
          zapisz={(w) => wykonaj(() => api.zapiszWalute(w))}
          usun={(w) => wykonaj(() => api.usunWalute(w.id))}
        />
      )}

      {zakladka === "numeracja" && (
        <Slownik<Numerator>
          wiersze={numeratory}
          kolumny={[
            {
              naglowek: "Przeznaczenie",
              pole: (n, zmien) => (
                <select value={n.przeznaczenie} onChange={(e) => zmien({ ...n, przeznaczenie: e.target.value as PrzeznaczenieNumeratora })}>
                  {przeznaczenia.map((p) => (
                    <option key={p} value={p}>
                      {p}
                    </option>
                  ))}
                </select>
              ),
            },
            {
              naglowek: "Format (np. FV/[Numer]/[Rok])",
              pole: (n, zmien) => <input value={n.format} onChange={(e) => zmien({ ...n, format: e.target.value })} />,
            },
            {
              naglowek: "Grupa licznika",
              pole: (n, zmien) => <input value={n.grupa ?? ""} onChange={(e) => zmien({ ...n, grupa: e.target.value || null })} />,
            },
          ]}
          nowy={() => ({ id: 0, przeznaczenie: "Faktura" as PrzeznaczenieNumeratora, format: "[Numer]", grupa: null })}
          zapisz={(n) => wykonaj(() => api.zapiszNumerator(n))}
          usun={(n) => wykonaj(() => api.usunNumerator(n.id))}
        />
      )}
    </section>
  );
}

interface KolumnaSlownika<T> {
  naglowek: string;
  pole: (wiersz: T, zmien: (nowy: T) => void) => React.ReactNode;
}

function Slownik<T extends { id: number }>({
  wiersze,
  kolumny,
  nowy,
  zapisz,
  usun,
}: {
  wiersze: T[];
  kolumny: KolumnaSlownika<T>[];
  nowy: () => T;
  zapisz: (wiersz: T) => void;
  usun: (wiersz: T) => void;
}) {
  const [robocze, setRobocze] = useState<Record<number, T>>({});
  const [nowyWiersz, setNowyWiersz] = useState<T | null>(null);

  useEffect(() => {
    setRobocze({});
    setNowyWiersz(null);
  }, [wiersze]);

  const wartosc = (w: T): T => robocze[w.id] ?? w;

  return (
    <table className="tabela">
      <thead>
        <tr>
          {kolumny.map((k) => (
            <th key={k.naglowek}>{k.naglowek}</th>
          ))}
          <th></th>
        </tr>
      </thead>
      <tbody>
        {wiersze.map((w) => (
          <tr key={w.id}>
            {kolumny.map((k) => (
              <td key={k.naglowek}>{k.pole(wartosc(w), (nowy) => setRobocze({ ...robocze, [w.id]: nowy }))}</td>
            ))}
            <td className="akcje">
              {robocze[w.id] && <button onClick={() => zapisz(robocze[w.id])}>Zapisz</button>}
              <button className="usun" onClick={() => usun(w)}>
                Usuń
              </button>
            </td>
          </tr>
        ))}
        {nowyWiersz && (
          <tr>
            {kolumny.map((k) => (
              <td key={k.naglowek}>{k.pole(nowyWiersz, setNowyWiersz)}</td>
            ))}
            <td className="akcje">
              <button onClick={() => zapisz(nowyWiersz)}>Zapisz</button>
              <button onClick={() => setNowyWiersz(null)}>Anuluj</button>
            </td>
          </tr>
        )}
      </tbody>
      <tfoot>
        <tr>
          <td colSpan={kolumny.length + 1}>
            <button onClick={() => setNowyWiersz(nowy())}>+ Dodaj</button>
          </td>
        </tr>
      </tfoot>
    </table>
  );
}
