import { useState } from "react";
import { Faktury } from "./views/Faktury";
import { Kontrahenci } from "./views/Kontrahenci";
import { Towary } from "./views/Towary";
import { Slowniki } from "./views/Slowniki";

type Widok = "sprzedaz" | "zakup" | "kontrahenci" | "towary" | "slowniki";

const pozycjeMenu: { id: Widok; etykieta: string }[] = [
  { id: "sprzedaz", etykieta: "Faktury sprzedaży" },
  { id: "zakup", etykieta: "Faktury zakupu" },
  { id: "kontrahenci", etykieta: "Kontrahenci" },
  { id: "towary", etykieta: "Towary i usługi" },
  { id: "slowniki", etykieta: "Słowniki" },
];

export default function App() {
  const [widok, setWidok] = useState<Widok>("sprzedaz");

  return (
    <div className="uklad">
      <nav className="menu">
        <h1 className="logo">ProFak</h1>
        {pozycjeMenu.map((p) => (
          <button
            key={p.id}
            className={widok === p.id ? "pozycja-menu aktywna" : "pozycja-menu"}
            onClick={() => setWidok(p.id)}
          >
            {p.etykieta}
          </button>
        ))}
      </nav>
      <main className="tresc">
        {widok === "sprzedaz" && <Faktury czySprzedaz={true} key="sprzedaz" />}
        {widok === "zakup" && <Faktury czySprzedaz={false} key="zakup" />}
        {widok === "kontrahenci" && <Kontrahenci />}
        {widok === "towary" && <Towary />}
        {widok === "slowniki" && <Slowniki />}
      </main>
    </div>
  );
}
