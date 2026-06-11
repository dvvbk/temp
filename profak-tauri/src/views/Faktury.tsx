import { useCallback, useEffect, useState } from "react";
import { api, FakturaListaWiersz, kwota, RodzajFaktury } from "../api";
import { FakturaEdytor } from "./FakturaEdytor";

const rodzajeSprzedazy: RodzajFaktury[] = ["Sprzedaż", "Proforma", "VatMarża", "Rachunek", "DowódWewnętrzny"];

export function Faktury({ czySprzedaz }: { czySprzedaz: boolean }) {
  const [wiersze, setWiersze] = useState<FakturaListaWiersz[]>([]);
  const [fraza, setFraza] = useState("");
  const [edytowanaId, setEdytowanaId] = useState<number | null>(null);
  const [nowyRodzaj, setNowyRodzaj] = useState<RodzajFaktury | null>(null);
  const [blad, setBlad] = useState("");
  const [komunikat, setKomunikat] = useState("");

  const odswiez = useCallback(() => {
    api
      .listaFaktur(czySprzedaz)
      .then(setWiersze)
      .catch((e) => setBlad(String(e)));
  }, [czySprzedaz]);

  useEffect(odswiez, [odswiez]);

  const przefiltrowane = wiersze.filter((w) => {
    const f = fraza.toLowerCase();
    if (!f) return true;
    return [w.numer, w.nazwa_nabywcy, w.nazwa_sprzedawcy, w.nip_nabywcy, w.nip_sprzedawcy, w.razem_brutto, w.rodzaj]
      .join(" ")
      .toLowerCase()
      .includes(f);
  });

  const korekta = async (id: number) => {
    try {
      const korektaId = await api.przygotujKorekte(id);
      setEdytowanaId(korektaId);
    } catch (e) {
      setBlad(String(e));
    }
  };

  const podobna = async (id: number) => {
    try {
      const kopiaId = await api.przygotujPodobna(id);
      setEdytowanaId(kopiaId);
    } catch (e) {
      setBlad(String(e));
    }
  };

  const pdf = async (id: number) => {
    try {
      const sciezka = await api.wydrukujFakture(id);
      setKomunikat(`Zapisano wydruk: ${sciezka}`);
    } catch (e) {
      setBlad(String(e));
    }
  };

  const usun = async (w: FakturaListaWiersz) => {
    if (!confirm(`Usunąć fakturę ${w.numer || "(bez numeru)"}?`)) return;
    try {
      await api.usunFakture(w.id);
      odswiez();
    } catch (e) {
      setBlad(String(e));
    }
  };

  if (edytowanaId !== null || nowyRodzaj !== null) {
    return (
      <FakturaEdytor
        fakturaId={edytowanaId}
        rodzaj={nowyRodzaj ?? "Sprzedaż"}
        zamknij={() => {
          setEdytowanaId(null);
          setNowyRodzaj(null);
          odswiez();
        }}
      />
    );
  }

  return (
    <section>
      <header className="naglowek-widoku">
        <h2>{czySprzedaz ? "Faktury sprzedaży" : "Faktury zakupu"}</h2>
        <input
          className="szukaj"
          placeholder="Szukaj..."
          value={fraza}
          onChange={(e) => setFraza(e.target.value)}
        />
        {czySprzedaz ? (
          rodzajeSprzedazy.map((r) => (
            <button key={r} onClick={() => setNowyRodzaj(r)}>
              + {r}
            </button>
          ))
        ) : (
          <button onClick={() => setNowyRodzaj("Zakup")}>+ Faktura zakupu</button>
        )}
      </header>
      {blad && <p className="blad">{blad}</p>}
      {komunikat && <p className="komunikat">{komunikat}</p>}
      <table className="tabela">
        <thead>
          <tr>
            <th>Numer</th>
            <th>Rodzaj</th>
            <th>Data wystawienia</th>
            <th>Termin płatności</th>
            <th>{czySprzedaz ? "Nabywca" : "Sprzedawca"}</th>
            <th className="num">Netto</th>
            <th className="num">VAT</th>
            <th className="num">Brutto</th>
            <th className="num">Do zapłaty</th>
            <th>Waluta</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {przefiltrowane.map((w) => (
            <tr key={w.id} onDoubleClick={() => setEdytowanaId(w.id)}>
              <td>{w.numer || <em>(robocza)</em>}</td>
              <td>{w.rodzaj}</td>
              <td>{w.data_wystawienia}</td>
              <td>{w.termin_platnosci}</td>
              <td>{czySprzedaz ? w.nazwa_nabywcy : w.nazwa_sprzedawcy}</td>
              <td className="num">{kwota(w.razem_netto)}</td>
              <td className="num">{kwota(w.razem_vat)}</td>
              <td className="num">{kwota(w.razem_brutto)}</td>
              <td className={Number(w.pozostalo_do_zaplaty) > 0 ? "num do-zaplaty" : "num"}>
                {kwota(w.pozostalo_do_zaplaty)}
              </td>
              <td>{w.waluta_skrot}</td>
              <td className="akcje">
                <button onClick={() => setEdytowanaId(w.id)}>Edytuj</button>
                <button onClick={() => pdf(w.id)}>PDF</button>
                <button onClick={() => korekta(w.id)}>Korekta</button>
                <button onClick={() => podobna(w.id)}>Podobna</button>
                <button className="usun" onClick={() => usun(w)}>
                  Usuń
                </button>
              </td>
            </tr>
          ))}
          {przefiltrowane.length === 0 && (
            <tr>
              <td colSpan={11} className="pusto">
                Brak faktur
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </section>
  );
}
