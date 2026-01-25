/**
 * Users Page
 * Displays user list (protected route)
 */

import { Show, For, createResource, Suspense } from "solid-js";
import { api, type User } from "../api";
import {
  Card,
  CardContent,
  Badge,
  Avatar,
  AvatarFallback,
  getInitials,
} from "../components/ui";
import { ProtectedRoute } from "../components/ProtectedRoute";
import { formatDate } from "../lib/utils";

function UserList() {
  const [users] = createResource(() => api.users.list());

  return (
    <div class="space-y-6">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Benutzer</h1>
        <p class="text-muted-foreground mt-2">
          Verwalte alle registrierten Benutzer.
        </p>
      </div>

      <Suspense
        fallback={
          <div class="flex items-center justify-center py-12">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
          </div>
        }
      >
        <Show
          when={users()}
          fallback={
            <Card>
              <CardContent class="py-12 text-center">
                <p class="text-muted-foreground">Keine Benutzer gefunden.</p>
              </CardContent>
            </Card>
          }
        >
          {(userList) => (
            <div class="grid gap-4">
              <For each={userList()}>
                {(user) => <UserCard user={user} />}
              </For>
            </div>
          )}
        </Show>
      </Suspense>
    </div>
  );
}

function UserCard(props: { user: User }) {
  return (
    <Card>
      <CardContent class="flex items-center gap-4 py-4">
        <Avatar>
          <AvatarFallback>
            {getInitials(props.user.name || props.user.email)}
          </AvatarFallback>
        </Avatar>
        
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <p class="font-medium truncate">
              {props.user.name || "Unbekannt"}
            </p>
            <div class="flex gap-1">
              <For each={props.user.roles || []}>
                {(role) => (
                  <Badge variant="outline" class="text-xs">
                    {role}
                  </Badge>
                )}
              </For>
            </div>
          </div>
          <p class="text-sm text-muted-foreground truncate">
            {props.user.email}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}

// Wrap in ProtectedRoute
export default function Users() {
  return (
    <ProtectedRoute>
      <UserList />
    </ProtectedRoute>
  );
}
