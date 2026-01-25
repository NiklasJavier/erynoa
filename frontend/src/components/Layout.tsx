/**
 * Layout Component
 * Main application layout with navigation
 */

import { type ParentComponent, Show } from "solid-js";
import { A } from "@solidjs/router";
import { useAuth, getUserDisplayName } from "../lib/auth";
import { Button, Avatar, AvatarFallback, getInitials } from "./ui";
import { ThemeToggle } from "./ThemeToggle";

export const Layout: ParentComponent = (props) => {
  const auth = useAuth();

  return (
    <div class="min-h-screen bg-background">
      {/* Navigation */}
      <nav class="border-b bg-card">
        <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
          <div class="flex h-16 items-center justify-between">
            {/* Logo & Nav Links */}
            <div class="flex items-center gap-8">
              <A href="/" class="flex items-center gap-2">
                <div class="h-8 w-8 rounded-lg bg-primary flex items-center justify-center">
                  <span class="text-primary-foreground font-bold text-lg">G</span>
                </div>
                <span class="font-semibold text-lg">Godstack</span>
              </A>

              <Show when={auth.isAuthenticated}>
                <div class="hidden md:flex items-center gap-4">
                  <A
                    href="/"
                    class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    activeClass="text-foreground"
                  >
                    Dashboard
                  </A>
                  <A
                    href="/users"
                    class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    activeClass="text-foreground"
                  >
                    Users
                  </A>
                  <A
                    href="/settings"
                    class="text-sm font-medium text-muted-foreground hover:text-foreground transition-colors"
                    activeClass="text-foreground"
                  >
                    Settings
                  </A>
                </div>
              </Show>
            </div>

            {/* User Menu */}
            <div class="flex items-center gap-4">
              <ThemeToggle />
              <Show
                when={auth.isAuthenticated}
                fallback={
                  <Button onClick={() => auth.login()}>
                    Login
                  </Button>
                }
              >
                <div class="flex items-center gap-3">
                  <span class="text-sm text-muted-foreground hidden sm:block">
                    {getUserDisplayName(auth.user)}
                  </span>
                  <Avatar size="sm">
                    <AvatarFallback>
                      {getInitials(getUserDisplayName(auth.user))}
                    </AvatarFallback>
                  </Avatar>
                  <Button variant="outline" size="sm" onClick={() => auth.logout()}>
                    Logout
                  </Button>
                </div>
              </Show>
            </div>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-8">
        {props.children}
      </main>

      {/* Footer */}
      <footer class="border-t bg-card mt-auto">
        <div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8 py-4">
          <p class="text-sm text-muted-foreground text-center">
            © 2026 Godstack. Built with SolidJS + Rust.
          </p>
        </div>
      </footer>
    </div>
  );
};

export default Layout;
