import { useCallback, useEffect, useState } from "react";
import { api, JednostkaMiary, kwota, nowyTowar, StawkaVat, Towar } from "../api";

export function Towary() {
  const [lista, setLista] = useState<Towar[]>([]);
  const [stawki, setStawki] = useState<StawkaVat[]>([]);
  const [jednostki, setJednostki] = useState<JednostkaMiary[]>([]);
  const [fraza, setFraza] = useState("");
  const [edytowany, setEdytowany] = useState<Towar | null>(null);
  const [blad, setBlad] = useState("");

  const odswiez = useCallback(() => {
    Promise.all([api.listaTowarow(), api.listaStawekVat(), api.listaJednostek()])
      .then(([t, s, j]) => {
        setLista(t);
        setStawki(s);
        setJednostki(j);
      })
      .catch((e) => setBlad(String(e)));
  }, []);

  useEffect(odswiez, [odswiez]);

  const zapisz = async () => {
    if (!edytowany) return;
    try {
      await api.zapiszTowar(edytowany);
      setEdytowany(null);
      odswiez();
    } catch (e) {
      setBlad(String(e));
    }
  };

  const usun = async (t: Towar) => {
    if (!confirm(`Usunąć "${t.nazwa}"?`)) return;
    try {
      await api.usunTowar(t.id);
      odswiez();
    } catch (e) {
      setBlad(String(e));
    }
  };

  const przefiltrowane = lista.filter((t) => t.nazwa.toLowerCase().includes(fraza.toLowerCase()));

  if (edytowany) {
    const zmien = (zmiany: Partial<Towar>) => setEdytowany({ ...edytowany, ...zmiany });
    return (
      <section>
        <header className="naglowek-widoku">
          <h2>{edytowany.id === 0 ? "Nowy towar/usługa" : edytowany.nazwa}</h2>
          <button className="glowny" onClick={zapisz}>
            Zapisz
          </button>
          <button onClick={() => setEdytowany(null)}>Anuluj</button>
        </header>
        {blad && <p className="blad">{blad}</p>}
        <div className="siatka-formularza">
          <fieldset>
            <legend>Dane</legend>
            <label>
              Nazwa
              <input value={edytowany.nazwa} onChange={(e) => zmien({ nazwa: e.target.value })} />
            </label>
            <label>
              Rodzaj
              <select value={edytowany.rodzaj} onChange={(e) => zmien({ rodzaj: e.target.value as Towar["rodzaj"] })}>
                <option value="Towar">Towar</option>
                <option value="Usługa">Usługa</option>
              </select>
            </label>
            <label>
              Sposób liczenia ceny
              <select
                value={edytowany.sposob_liczenia_ceny}
                onChange={(e) => zmien({ sposob_liczenia_ceny: e.target.value as Towar["sposob_liczenia_ceny"] })}
              >
                <option value="WedługNetto">Według ceny netto</option>
                <option value="WedługBrutto">Według ceny brutto</option>
                <option value="NarzutKwotowy">Narzut kwotowy do ceny zakupu</option>
                <option value="NarzutProcentowy">Narzut procentowy do ceny zakupu</option>
              </select>
            </label>
            <label>
              Cena netto
              <input className="num" value={edytowany.cena_netto} onChange={(e) => zmien({ cena_netto: e.target.value || "0" })} />
            </label>
            <label>
              Cena brutto
              <input className="num" value={edytowany.cena_brutto} onChange={(e) => zmien({ cena_brutto: e.target.value || "0" })} />
            </label>
            <label>
              Stawka VAT
              <select
                value={edytowany.stawka_vat_id ?? ""}
                onChange={(e) => zmien({ stawka_vat_id: e.target.value ? Number(e.target.value) : null })}
              >
                <option value="">—</option>
                {stawki.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.skrot}
                  </option>
                ))}
              </select>
            </label>
            <label>
              Jednostka miary
              <select
                value={edytowany.jednostka_miary_id ?? ""}
                onChange={(e) => zmien({ jednostka_miary_id: e.target.value ? Number(e.target.value) : null })}
              >
                <option value="">—</option>
                {jednostki.map((j) => (
                  <option key={j.id} value={j.id}>
                    {j.skrot}
                  </option>
                ))}
              </select>
            </label>
            <label>
              GTU
              <input className="num" value={edytowany.gtu} onChange={(e) => zmien({ gtu: Number(e.target.value) || 0 })} />
            </label>
            <label className="pole-wyboru">
              <input type="checkbox" checked={edytowany.czy_archiwalny} onChange={(e) => zmien({ czy_archiwalny: e.target.checked })} />
              Archiwalny
            </label>
          </fieldset>
        </div>
      </section>
    );
  }

  return (
    <section>
      <header className="naglowek-widoku">
        <h2>Towary i usługi</h2>
        <input className="szukaj" placeholder="Szukaj..." value={fraza} onChange={(e) => setFraza(e.target.value)} />
        <button className="glowny" onClick={() => setEdytowany(nowyTowar())}>
          + Nowy
        </button>
      </header>
      {blad && <p className="blad">{blad}</p>}
      <table className="tabela">
        <thead>
          <tr>
            <th>Nazwa</th>
            <th>Rodzaj</th>
            <th className="num">Cena netto</th>
            <th className="num">Cena brutto</th>
            <th>VAT</th>
            <th>JM</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {przefiltrowane.map((t) => (
            <tr key={t.id} onDoubleClick={() => setEdytowany(t)}>
              <td>{t.nazwa}</td>
              <td>{t.rodzaj}</td>
              <td className="num">{kwota(t.cena_netto)}</td>
              <td className="num">{kwota(t.cena_brutto)}</td>
              <td>{stawki.find((s) => s.id === t.stawka_vat_id)?.skrot ?? ""}</td>
              <td>{jednostki.find((j) => j.id === t.jednostka_miary_id)?.skrot ?? ""}</td>
              <td className="akcje">
                <button onClick={() => setEdytowany(t)}>Edytuj</button>
                <button className="usun" onClick={() => usun(t)}>
                  Usuń
                </button>
              </td>
            </tr>
          ))}
          {przefiltrowane.length === 0 && (
            <tr>
              <td colSpan={7} className="pusto">
                Brak towarów
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </section>
  );
}
