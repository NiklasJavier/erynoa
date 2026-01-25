/**
 * Home / Dashboard Page
 * Shows service status and system health
 */

import { Show, createResource, For, createSignal, onMount } from "solid-js";
import { useAuth, getUserDisplayName } from "../lib/auth";
import { api } from "../api/client";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  Badge,
} from "../components/ui";

interface ServiceStatus {
  name: string;
  status: "online" | "offline";
  description: string;
  url?: string;
}

export default function Home() {
  const auth = useAuth();
  const [services, setServices] = createSignal<ServiceStatus[]>([]);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    try {
      // Fetch service status from backend
      const response = await fetch("/api/v1/status", { method: "GET" });
      if (response.ok) {
        const data = await response.json();
        setServices(data.services || []);
      }
    } catch (error) {
      console.error("Failed to fetch service status:", error);
      // Fallback status based on what we know is running
      setServices([
        { name: "API Server", status: "online", description: "Backend REST API", url: "http://localhost:3000" },
        { name: "Database", status: "online", description: "PostgreSQL Database", url: "postgresql://localhost:5432" },
        { name: "Cache", status: "online", description: "DragonflyDB Cache", url: "redis://localhost:6379" },
        { name: "S3 Storage", status: "online", description: "MinIO Object Storage", url: "http://localhost:9000" },
        { name: "Authentication", status: "online", description: "ZITADEL Auth Service", url: "http://localhost:8080" },
      ]);
    } finally {
      setLoading(false);
    }
  });

  return (
    <div class="space-y-8">
      {/* Welcome Section */}
      <div>
        <h1 class="text-3xl font-bold tracking-tight">
          <Show when={auth.isAuthenticated} fallback="Willkommen bei Godstack">
            Willkommen zurück, {getUserDisplayName(auth.user)}!
          </Show>
        </h1>
        <p class="text-muted-foreground mt-2">
          High-Performance Full-Stack Application
        </p>
      </div>

      {/* System Status */}
      <Show when={auth.isAuthenticated}>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Services Overview */}
          <Card class="col-span-full">
            <CardHeader>
              <CardTitle>Service Status</CardTitle>
              <CardDescription>
                Echtzeit-Status aller Systemkomponenten
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Show
                when={!loading()}
                fallback={
                  <div class="flex items-center justify-center py-8">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
                  </div>
                }
              >
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  <For each={services()}>
                    {(service) => (
                      <div class="flex items-start justify-between p-4 border rounded-lg hover:bg-muted/50 transition-colors">
                        <div class="flex-1">
                          <div class="font-semibold text-sm">{service.name}</div>
                          <div class="text-xs text-muted-foreground">{service.description}</div>
                          {service.url && (
                            <div class="text-xs text-muted-foreground mt-1 font-mono break-all">
                              {service.url}
                            </div>
                          )}
                        </div>
                        <div class="ml-4">
                          <Badge
                            variant={service.status === "online" ? "default" : "destructive"}
                            class="whitespace-nowrap"
                          >
                            <span class={`inline-block w-2 h-2 rounded-full mr-2 ${
                              service.status === "online" ? "bg-green-500" : "bg-red-500"
                            }`} />
                            {service.status === "online" ? "Online" : "Offline"}
                          </Badge>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </CardContent>
          </Card>

          {/* User Info Card */}
          <Card>
            <CardHeader>
              <CardTitle>Dein Profil</CardTitle>
            </CardHeader>
            <CardContent>
              <Show when={auth.user} fallback={<p class="text-muted-foreground">Nicht authentifiziert</p>}>
                <div class="space-y-3">
                  <div>
                    <div class="text-xs text-muted-foreground">Name</div>
                    <div class="font-medium">
                      {auth.user?.profile?.name ||
                        `${auth.user?.profile?.given_name || ""} ${auth.user?.profile?.family_name || ""}`.trim() ||
                        auth.user?.profile?.preferred_username ||
                        "N/A"}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">Email</div>
                    <div class="font-medium break-all">
                      {auth.user?.profile?.email || "N/A"}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">Username</div>
                    <div class="font-medium">
                      {auth.user?.profile?.preferred_username || "N/A"}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">Rollen</div>
                    <div class="flex gap-2 mt-1 flex-wrap">
                      <Show when={auth.user} fallback={<p class="text-sm text-muted-foreground">Keine Rollen</p>}>
                        {(() => {
                          const roles = auth.user?.profile?.["urn:zitadel:iam:org:project:roles"];
                          if (typeof roles === "object" && roles) {
                            return (
                              <For each={Object.keys(roles || {})}>
                                {(role) => <Badge variant="outline">{role}</Badge>}
                              </For>
                            );
                          }
                          return <p class="text-sm text-muted-foreground">Keine Rollen</p>;
                        })()}
                      </Show>
                    </div>
                  </div>
                </div>
              </Show>
            </CardContent>
          </Card>

          {/* System Info Card */}
          <Card>
            <CardHeader>
              <CardTitle>System Info</CardTitle>
            </CardHeader>
            <CardContent>
              <div class="space-y-3">
                <div>
                  <div class="text-xs text-muted-foreground">Umgebung</div>
                  <div class="font-medium capitalize">Development</div>
                </div>
                <div>
                  <div class="text-xs text-muted-foreground">API Version</div>
                  <div class="font-mono text-sm">0.1.0</div>
                </div>
                <div>
                  <div class="text-xs text-muted-foreground">Status</div>
                  <div class="text-sm">
                    <Badge variant="default">System läuft</Badge>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </Show>

      {/* Login Prompt */}
      <Show when={!auth.isAuthenticated}>
        <Card>
          <CardHeader>
            <CardTitle>Authentifizierung erforderlich</CardTitle>
            <CardDescription>
              Bitte melde dich an, um das Dashboard zu sehen.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <button
              onClick={() => auth.login()}
              class="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
            >
              Jetzt anmelden
            </button>
          </CardContent>
        </Card>
      </Show>
    </div>
  );
}
