import { useEffect, useState } from "react";
import { commands } from "./bindings";
import { t } from "./messages";

// The walking skeleton's empty window. The real views arrive in later slices;
// the styled UI comes in M2.
export function App() {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    void commands.appVersion().then(setVersion);
  }, []);

  return (
    <main>
      <h1>{t("app.name")}</h1>
      {version !== null && <p>{t("app.version", { version })}</p>}
    </main>
  );
}
