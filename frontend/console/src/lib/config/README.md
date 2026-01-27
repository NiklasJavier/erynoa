# Deklarative Config-Struktur

Diese Dokumentation beschreibt, wie man schnell neue Seiten und Navigationselemente hinzufügen kann.

## Neue Seite hinzufügen

### 1. Page Metadata in `lib/config/pages.ts` hinzufügen

```typescript
export const pageConfig: Record<string, PageMeta> = {
  // ... bestehende Seiten
  
  '/reports': {
    title: 'Reports',
    description: 'View and generate reports',
    showHeader: true,  // Optional: Standard ist true
  },
};
```

### 2. Navigation Item in `lib/config/navigation.ts` hinzufügen

```typescript
import FileText from 'lucide-svelte/icons/file-text';

export const navigationConfig: NavConfig = {
  // ...
  groups: [
    {
      label: 'Platform',
      items: [
        // ... bestehende Items
        { 
          title: 'Reports', 
          url: '/reports', 
          icon: FileText,
          badge: 'New',        // Optional: zeigt Badge an
          disabled: false,     // Optional: deaktiviert Link
        },
      ],
    },
  ],
};
```

### 3. Route-Datei erstellen (`routes/reports/+page.svelte`)

```svelte
<script lang="ts">
  import PageContent from '$lib/components/PageContent.svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
</script>

<!-- Header (Titel + Description) kommt automatisch aus pageConfig -->
<PageContent>
  {#snippet headerActions()}
    <Button>Generate Report</Button>
  {/snippet}
  
  <!-- Page Content hier -->
  <Card.Root>
    <Card.Content>
      ...
    </Card.Content>
  </Card.Root>
</PageContent>
```

---

## Neue Kategorie/Gruppe hinzufügen

In `lib/config/navigation.ts`:

```typescript
export const navigationConfig: NavConfig = {
  groups: [
    // Bestehende Platform-Gruppe
    {
      label: 'Platform',
      items: [/* ... */],
    },
    
    // NEUE Gruppe hinzufügen
    {
      label: 'Analytics', 
      items: [
        { title: 'Overview', url: '/analytics', icon: ChartBar },
        { title: 'Reports', url: '/analytics/reports', icon: FileText },
        { title: 'Exports', url: '/analytics/exports', icon: Download },
      ],
    },
  ],
};
```

---

## Dropdown/Sub-Items hinzufügen

Für Navigation mit aufklappbaren Sub-Items:

```typescript
import Settings from 'lucide-svelte/icons/settings';
import Cog from 'lucide-svelte/icons/cog';
import Shield from 'lucide-svelte/icons/shield';
import CreditCard from 'lucide-svelte/icons/credit-card';

export const navigationConfig: NavConfig = {
  groups: [
    {
      label: 'Administration',
      items: [
        { 
          title: 'Settings', 
          icon: Settings,
          defaultOpen: true,  // Automatisch aufgeklappt
          children: [
            { title: 'General', url: '/settings/general', icon: Cog },
            { title: 'Security', url: '/settings/security', icon: Shield },
            { title: 'Billing', url: '/settings/billing', icon: CreditCard },
          ],
        },
      ],
    },
  ],
};
```

---

## Struktur-Übersicht

```
lib/config/
├── index.ts          # Zentrale Exports
├── app.ts            # App-Config (URLs, Version, etc.)
├── navigation.ts     # Sidebar-Navigation
├── pages.ts          # Page-Metadaten (Titel, Beschreibung)
└── readme.md         # Diese Dokumentation

lib/components/
├── PageContent.svelte       # Auto-Header aus pageConfig
└── dashboard/
    └── AppSidebar.svelte    # Dynamisch aus navigationConfig
```

## NavItem Optionen

| Property   | Type             | Description                    |
|------------|------------------|--------------------------------|
| `title`    | string           | Anzeigename                    |
| `url`      | string           | Route-Pfad                     |
| `icon`     | Component        | Lucide Icon                    |
| `badge`    | string (opt)     | Badge-Text (z.B. "New", "3")   |
| `disabled` | boolean (opt)    | Deaktiviert Link               |

## NavItemWithChildren Optionen (Dropdown)

| Property       | Type        | Description                       |
|----------------|-------------|-----------------------------------|
| `title`        | string      | Anzeigename des Dropdown-Triggers |
| `icon`         | Component   | Lucide Icon                       |
| `children`     | NavItem[]   | Sub-Items                         |
| `defaultOpen`  | boolean     | Anfangs aufgeklappt               |
| `url`          | string      | Optional: Direktlink              |

## PageMeta Optionen

| Property        | Type        | Description                           |
|-----------------|-------------|---------------------------------------|
| `title`         | string      | Seitentitel (H1)                      |
| `description`   | string      | Beschreibung unter Titel              |
| `showHeader`    | boolean     | Header anzeigen (default: true)       |
| `documentTitle` | string      | Custom Browser-Tab-Titel              |

## PageContent Props

| Prop            | Type       | Description                           |
|-----------------|------------|---------------------------------------|
| `title`         | string     | Überschreibt Config-Titel             |
| `description`   | string     | Überschreibt Config-Beschreibung      |
| `showHeader`    | boolean    | Header anzeigen                       |
| `loading`       | boolean    | Zeigt Loading-Spinner                 |
| `error`         | string     | Zeigt Error-Alert                     |
| `onRetry`       | () => void | Retry-Button Callback                 |
| `headerActions` | Snippet    | Buttons rechts im Header              |
| `children`      | Snippet    | Page-Inhalt                           |

---

## Beispiel: Loading & Error States

```svelte
<script lang="ts">
  let loading = $state(true);
  let error = $state<string | null>(null);
  
  async function loadData() {
    loading = true;
    error = null;
    try {
      // fetch data...
    } catch (e) {
      error = e.message;
    } finally {
      loading = false;
    }
  }
</script>

<PageContent {loading} {error} onRetry={loadData}>
  <!-- Wird nur gerendert wenn !loading && !error -->
  <Card.Root>...</Card.Root>
</PageContent>
```

## Automatischer Document Title

Der Browser-Tab-Titel wird automatisch aus `pageConfig` generiert:

- `/` → "Erynoa"
- `/users` → "Users | Erynoa"
- `/settings` → "Settings | Erynoa"

Custom Title möglich:
```typescript
'/special': {
  title: 'Special Page',
  documentTitle: 'Mein Custom Titel'  // Überschreibt Standard
}
```