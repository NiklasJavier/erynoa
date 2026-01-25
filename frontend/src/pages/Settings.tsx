/**
 * Settings Page
 * User settings and preferences (protected route)
 */

import { Show } from "solid-js";
import { useAuth, getUserDisplayName } from "../lib/auth";
import { useTheme } from "../lib/theme";
import { getZitadelConsoleUrl } from "../lib/service-urls";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
  Button,
  Input,
  Badge,
} from "../components/ui";
import { ThemeSelect } from "../components/ThemeToggle";
import { ProtectedRoute } from "../components/ProtectedRoute";

function SettingsContent() {
  const auth = useAuth();
  const { theme } = useTheme();

  return (
    <div class="space-y-6">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Einstellungen</h1>
        <p class="text-muted-foreground mt-2">
          Verwalte dein Konto und Präferenzen.
        </p>
      </div>

      {/* Appearance Card */}
      <Card>
        <CardHeader>
          <CardTitle>Erscheinungsbild</CardTitle>
          <CardDescription>
            Passe das Aussehen der Anwendung an.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex justify-between items-center">
            <div>
              <p class="font-medium">Theme</p>
              <p class="text-sm text-muted-foreground">
                Wähle zwischen Hell, Dunkel oder Systemeinstellung
              </p>
            </div>
            <ThemeSelect />
          </div>
          <p class="text-xs text-muted-foreground">
            Aktuell: {theme() === "system" ? "Systemeinstellung" : theme() === "dark" ? "Dunkel" : "Hell"}
          </p>
        </CardContent>
      </Card>

      {/* Profile Card */}
      <Card>
        <CardHeader>
          <CardTitle>Profil</CardTitle>
          <CardDescription>
            Deine Profilinformationen aus ZITADEL.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="grid gap-4 md:grid-cols-2">
            <div class="space-y-2">
              <label class="text-sm font-medium">Name</label>
              <Input 
                value={getUserDisplayName(auth.user)} 
                disabled 
                class="bg-muted"
              />
            </div>
            <div class="space-y-2">
              <label class="text-sm font-medium">E-Mail</label>
              <Input 
                value={auth.user?.profile?.email || ""} 
                disabled 
                class="bg-muted"
              />
            </div>
          </div>
          <p class="text-xs text-muted-foreground">
            Profildaten werden in ZITADEL verwaltet. 
            <a 
              href={getZitadelConsoleUrl()} 
              target="_blank" 
              class="text-primary hover:underline ml-1"
            >
              ZITADEL Console öffnen →
            </a>
          </p>
        </CardContent>
      </Card>

      {/* Session Card */}
      <Card>
        <CardHeader>
          <CardTitle>Session</CardTitle>
          <CardDescription>
            Informationen zu deiner aktuellen Sitzung.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex justify-between items-center">
            <span class="text-sm text-muted-foreground">Session ID</span>
            <code class="text-xs bg-muted px-2 py-1 rounded font-mono">
              {auth.user?.session_state?.slice(0, 16)}...
            </code>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-sm text-muted-foreground">Token gültig bis</span>
            <Show when={auth.user?.expires_at}>
              <Badge variant="outline">
                {new Date(auth.user!.expires_at! * 1000).toLocaleString("de-DE")}
              </Badge>
            </Show>
          </div>
          <div class="flex justify-between items-center">
            <span class="text-sm text-muted-foreground">Issuer</span>
            <code class="text-xs bg-muted px-2 py-1 rounded font-mono truncate max-w-[200px]">
              {auth.user?.profile?.iss}
            </code>
          </div>
        </CardContent>
        <CardFooter>
          <Button variant="destructive" onClick={() => auth.logout()}>
            Abmelden
          </Button>
        </CardFooter>
      </Card>

      {/* Security Card */}
      <Card>
        <CardHeader>
          <CardTitle>Sicherheit</CardTitle>
          <CardDescription>
            Verwalte deine Sicherheitseinstellungen.
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="flex justify-between items-center">
            <div>
              <p class="font-medium">Passwort ändern</p>
              <p class="text-sm text-muted-foreground">
                Ändere dein Passwort in ZITADEL
              </p>
            </div>
            <a 
              href={getZitadelConsoleUrl("users/me?id=security")}
              target="_blank"
            >
              <Button variant="outline">
                Ändern
              </Button>
            </a>
          </div>
          <div class="flex justify-between items-center">
            <div>
              <p class="font-medium">Zwei-Faktor-Authentifizierung</p>
              <p class="text-sm text-muted-foreground">
                Aktiviere zusätzliche Sicherheit
              </p>
            </div>
            <a 
              href={getZitadelConsoleUrl("users/me?id=security")}
              target="_blank"
            >
              <Button variant="outline">
                Verwalten
              </Button>
            </a>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// Wrap in ProtectedRoute
export default function Settings() {
  return (
    <ProtectedRoute>
      <SettingsContent />
    </ProtectedRoute>
  );
}
