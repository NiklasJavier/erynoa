/**
 * Protected Route Component
 * Redirects to login if not authenticated
 */

import { type ParentComponent, Show, createEffect } from "solid-js";
import { useNavigate } from "@solidjs/router";
import { useAuth } from "../lib/auth";

export const ProtectedRoute: ParentComponent = (props) => {
  const auth = useAuth();
  const navigate = useNavigate();

  createEffect(() => {
    if (!auth.isLoading && !auth.isAuthenticated) {
      // Redirect to home/login
      navigate("/", { replace: true });
    }
  });

  return (
    <Show
      when={!auth.isLoading}
      fallback={
        <div class="flex items-center justify-center min-h-[50vh]">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
        </div>
      }
    >
      <Show
        when={auth.isAuthenticated}
        fallback={
          <div class="text-center py-12">
            <p class="text-muted-foreground">Please log in to access this page.</p>
          </div>
        }
      >
        {props.children}
      </Show>
    </Show>
  );
};

export default ProtectedRoute;
