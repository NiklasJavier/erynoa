/**
 * 404 Not Found Page
 */

import { A } from "@solidjs/router";
import { Button } from "../components/ui";

export default function NotFound() {
  return (
    <div class="flex flex-col items-center justify-center min-h-[60vh] gap-6 text-center">
      <div class="space-y-2">
        <h1 class="text-6xl font-bold text-muted-foreground">404</h1>
        <h2 class="text-2xl font-semibold">Seite nicht gefunden</h2>
        <p class="text-muted-foreground max-w-md">
          Die angeforderte Seite existiert nicht oder wurde verschoben.
        </p>
      </div>
      <A href="/">
        <Button>Zurück zur Startseite</Button>
      </A>
    </div>
  );
}
