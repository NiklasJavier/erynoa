/**
 * Home / Dashboard Page
 * Shows service status and system health
 */

import { Show, For, createSignal, onMount } from "solid-js";
import { useAuth, getUserDisplayName } from "../lib/auth";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  Badge,
} from "../components/ui";
import { infoClient, healthClient } from "../api/connect/services";
import { GetInfoRequest } from "../api/info";
import { ReadyRequest } from "../api/health";
import { logger } from "../lib/logger";

interface ServiceStatus {
  name: string;
  status: "online" | "offline";
  description: string;
  url?: string;
  latency?: number;
  message?: string;
}

export default function Home() {
  const auth = useAuth();
  const [services, setServices] = createSignal<ServiceStatus[]>([]);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    try {
      // Fetch info and health status from backend using Connect-RPC
      const [infoResponse, readyResponse] = await Promise.all([
        infoClient.getInfo(new GetInfoRequest({})),
        healthClient.ready(new ReadyRequest({})),
      ]);
      
      // Map ready response to service status with real health data
      const serviceList: ServiceStatus[] = [
        {
          name: "API Server",
          status: readyResponse.ready ? "online" : "offline",
          description: "Backend Connect-RPC API",
          url: infoResponse.urls?.api,
          latency: 0, // API latency is the request itself
          message: readyResponse.ready ? "operational" : "unavailable",
        },
        {
          name: "Database",
          status: readyResponse.database?.healthy ? "online" : "offline",
          description: "PostgreSQL Database",
          latency: readyResponse.database ? Number(readyResponse.database.latencyMs) : undefined,
          message: readyResponse.database?.message || "unknown",
        },
        {
          name: "Cache",
          status: readyResponse.cache?.healthy ? "online" : "offline",
          description: "DragonflyDB Cache",
          latency: readyResponse.cache ? Number(readyResponse.cache.latencyMs) : undefined,
          message: readyResponse.cache?.message || "unknown",
        },
        {
          name: "S3 Storage",
          status: readyResponse.storage?.healthy ? "online" : "offline",
          description: "MinIO Object Storage",
          latency: readyResponse.storage ? Number(readyResponse.storage.latencyMs) : undefined,
          message: readyResponse.storage?.message || "unknown",
        },
        {
          name: "Authentication",
          status: readyResponse.auth?.healthy ? "online" : "offline",
          description: "ZITADEL Auth Service",
          url: infoResponse.auth?.issuer,
          latency: readyResponse.auth ? Number(readyResponse.auth.latencyMs) : undefined,
          message: readyResponse.auth?.message || "unknown",
        },
      ];
      setServices(serviceList);
    } catch (error) {
      logger.error("Failed to fetch service status", error instanceof Error ? error : new Error(String(error)), {
        component: "Home",
        action: "fetchServiceStatus",
      });
      
      const errorMessage = error instanceof Error ? error.message : String(error);
      
      // Fallback status with error message
      setServices([
        { name: "API Server", status: "offline", description: "Backend Connect-RPC API", message: `Error: ${errorMessage.substring(0, 50)}` },
        { name: "Database", status: "offline", description: "PostgreSQL Database", message: "Unable to check" },
        { name: "Cache", status: "offline", description: "DragonflyDB Cache", message: "Unable to check" },
        { name: "S3 Storage", status: "offline", description: "MinIO Object Storage", message: "Unable to check" },
        { name: "Authentication", status: "offline", description: "ZITADEL Auth Service", message: "Unable to check" },
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
                          {service.message && (
                            <div class="text-xs text-muted-foreground mt-1">
                              Status: {service.message}
                            </div>
                          )}
                          {service.latency !== undefined && service.latency > 0 && (
                            <div class="text-xs text-muted-foreground mt-1">
                              Response Time: {service.latency}ms
                            </div>
                          )}
                          {service.url && (
                            <div class="text-xs text-muted-foreground mt-1 font-mono break-all">
                              {service.url}
                            </div>
                          )}
                        </div>
                        <div class="ml-4 flex flex-col items-end gap-2">
                          <Badge
                            variant={service.status === "online" ? "default" : "destructive"}
                            class="whitespace-nowrap"
                          >
                            <span class={`inline-block w-2 h-2 rounded-full mr-2 ${
                              service.status === "online" ? "bg-green-500" : "bg-red-500"
                            }`} />
                            {service.status === "online" ? "Online" : "Offline"}
                          </Badge>
                          {service.latency !== undefined && service.latency > 0 && (
                            <div class="text-xs text-muted-foreground">
                              {service.latency}ms
                            </div>
                          )}
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
