import { useCallback, useEffect, useState } from "react";
import { api, Kontrahent, nowyKontrahent } from "../api";

export function Kontrahenci() {
  const [lista, setLista] = useState<Kontrahent[]>([]);
  const [fraza, setFraza] = useState("");
  const [edytowany, setEdytowany] = useState<Kontrahent | null>(null);
  const [blad, setBlad] = useState("");

  const odswiez = useCallback(() => {
    api.listaKontrahentow().then(setLista).catch((e) => setBlad(String(e)));
  }, []);

  useEffect(odswiez, [odswiez]);

  const zapisz = async () => {
    if (!edytowany) return;
    try {
      await api.zapiszKontrahenta(edytowany);
      setEdytowany(null);
      odswiez();
    } catch (e) {
      setBlad(String(e));
    }
  };

  const usun = async (k: Kontrahent) => {
    if (!confirm(`Usunąć kontrahenta ${k.nazwa}?`)) return;
    try {
      await api.usunKontrahenta(k.id);
      odswiez();
    } catch (e) {
      setBlad(String(e));
    }
  };

  const przefiltrowani = lista.filter((k) =>
    [k.nazwa, k.pelna_nazwa, k.nip, k.adres_rejestrowy, k.email].join(" ").toLowerCase().includes(fraza.toLowerCase()),
  );

  if (edytowany) {
    const zmien = (zmiany: Partial<Kontrahent>) => setEdytowany({ ...edytowany, ...zmiany });
    return (
      <section>
        <header className="naglowek-widoku">
          <h2>{edytowany.id === 0 ? "Nowy kontrahent" : edytowany.nazwa}</h2>
          <button className="glowny" onClick={zapisz}>
            Zapisz
          </button>
          <button onClick={() => setEdytowany(null)}>Anuluj</button>
        </header>
        {blad && <p className="blad">{blad}</p>}
        <div className="siatka-formularza">
          <fieldset>
            <legend>Dane podstawowe</legend>
            <label>
              Nazwa skrócona
              <input value={edytowany.nazwa} onChange={(e) => zmien({ nazwa: e.target.value })} />
            </label>
            <label>
              Pełna nazwa
              <input value={edytowany.pelna_nazwa} onChange={(e) => zmien({ pelna_nazwa: e.target.value })} />
            </label>
            <label>
              NIP
              <input value={edytowany.nip} onChange={(e) => zmien({ nip: e.target.value })} />
            </label>
            <label>
              Adres rejestrowy
              <textarea value={edytowany.adres_rejestrowy} onChange={(e) => zmien({ adres_rejestrowy: e.target.value })} />
            </label>
            <label className="pole-wyboru">
              <input type="checkbox" checked={edytowany.czy_podmiot} onChange={(e) => zmien({ czy_podmiot: e.target.checked })} />
              Mój podmiot (sprzedawca na fakturach)
            </label>
            <label className="pole-wyboru">
              <input type="checkbox" checked={edytowany.czy_archiwalny} onChange={(e) => zmien({ czy_archiwalny: e.target.checked })} />
              Archiwalny
            </label>
          </fieldset>
          <fieldset>
            <legend>Kontakt i bank</legend>
            <label>
              Telefon
              <input value={edytowany.telefon} onChange={(e) => zmien({ telefon: e.target.value })} />
            </label>
            <label>
              E-mail
              <input value={edytowany.email} onChange={(e) => zmien({ email: e.target.value })} />
            </label>
            <label>
              Rachunek bankowy
              <input value={edytowany.rachunek_bankowy} onChange={(e) => zmien({ rachunek_bankowy: e.target.value })} />
            </label>
            <label>
              Nazwa banku
              <input value={edytowany.nazwa_banku} onChange={(e) => zmien({ nazwa_banku: e.target.value })} />
            </label>
            <label>
              Uwagi wewnętrzne
              <textarea value={edytowany.uwagi_wewnetrzne} onChange={(e) => zmien({ uwagi_wewnetrzne: e.target.value })} />
            </label>
            <label>
              Uwagi na fakturach
              <textarea value={edytowany.uwagi_publiczne} onChange={(e) => zmien({ uwagi_publiczne: e.target.value })} />
            </label>
          </fieldset>
        </div>
      </section>
    );
  }

  return (
    <section>
      <header className="naglowek-widoku">
        <h2>Kontrahenci</h2>
        <input className="szukaj" placeholder="Szukaj..." value={fraza} onChange={(e) => setFraza(e.target.value)} />
        <button className="glowny" onClick={() => setEdytowany(nowyKontrahent())}>
          + Nowy kontrahent
        </button>
      </header>
      {blad && <p className="blad">{blad}</p>}
      <table className="tabela">
        <thead>
          <tr>
            <th>Nazwa</th>
            <th>NIP</th>
            <th>Adres</th>
            <th>E-mail</th>
            <th>Telefon</th>
            <th>Podmiot</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {przefiltrowani.map((k) => (
            <tr key={k.id} onDoubleClick={() => setEdytowany(k)}>
              <td>{k.nazwa}</td>
              <td>{k.nip}</td>
              <td>{k.adres_rejestrowy}</td>
              <td>{k.email}</td>
              <td>{k.telefon}</td>
              <td>{k.czy_podmiot ? "Tak" : ""}</td>
              <td className="akcje">
                <button onClick={() => setEdytowany(k)}>Edytuj</button>
                <button className="usun" onClick={() => usun(k)}>
                  Usuń
                </button>
              </td>
            </tr>
          ))}
          {przefiltrowani.length === 0 && (
            <tr>
              <td colSpan={7} className="pusto">
                Brak kontrahentów
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </section>
  );
}
