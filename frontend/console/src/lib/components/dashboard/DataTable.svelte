<script lang="ts">
  import { Badge } from "@erynoa/ui/components/badge";
  import { Button } from "@erynoa/ui/components/button";
  import * as DropdownMenu from "@erynoa/ui/components/dropdown-menu";
  import * as Table from "@erynoa/ui/components/table";
  import * as Tabs from "@erynoa/ui/components/tabs";
  import ArrowUpDown from "lucide-svelte/icons/arrow-up-down";
  import ChevronLeft from "lucide-svelte/icons/chevron-left";
  import ChevronRight from "lucide-svelte/icons/chevron-right";
  import MoreHorizontal from "lucide-svelte/icons/more-horizontal";

  interface User {
    id: string;
    name: string;
    email: string;
    status: "Active" | "Inactive" | "Pending";
    role: string;
    lastActive: string;
  }

  const users: User[] = [
    {
      id: "1",
      name: "Olivia Martin",
      email: "olivia.martin@email.com",
      status: "Active",
      role: "Admin",
      lastActive: "2 min ago",
    },
    {
      id: "2",
      name: "Jackson Lee",
      email: "jackson.lee@email.com",
      status: "Active",
      role: "User",
      lastActive: "5 min ago",
    },
    {
      id: "3",
      name: "Isabella Nguyen",
      email: "isabella.nguyen@email.com",
      status: "Pending",
      role: "User",
      lastActive: "1 hour ago",
    },
    {
      id: "4",
      name: "William Kim",
      email: "will@email.com",
      status: "Active",
      role: "User",
      lastActive: "3 hours ago",
    },
    {
      id: "5",
      name: "Sofia Davis",
      email: "sofia.davis@email.com",
      status: "Inactive",
      role: "User",
      lastActive: "2 days ago",
    },
  ];

  let activeTab = $state("all");

  const filteredUsers = $derived(() => {
    if (activeTab === "all") return users;
    return users.filter((u) => u.status.toLowerCase() === activeTab);
  });

  function getStatusVariant(
    status: string,
  ): "default" | "secondary" | "outline" {
    switch (status) {
      case "Active":
        return "default";
      case "Inactive":
        return "secondary";
      case "Pending":
        return "outline";
      default:
        return "secondary";
    }
  }
</script>

<div class="space-y-4">
  <Tabs.Root value={activeTab} onValueChange={(v) => v && (activeTab = v)}>
    <Tabs.List>
      <Tabs.Trigger value="all">All Users</Tabs.Trigger>
      <Tabs.Trigger value="active">Active</Tabs.Trigger>
      <Tabs.Trigger value="inactive">Inactive</Tabs.Trigger>
      <Tabs.Trigger value="pending">Pending</Tabs.Trigger>
    </Tabs.List>
  </Tabs.Root>

  <div class="rounded-md border">
    <Table.Root>
      <Table.Header>
        <Table.Row>
          <Table.Head class="w-[200px]">
            <Button variant="ghost" class="-ml-4 h-8">
              Name
              <ArrowUpDown class="ml-2 h-4 w-4" />
            </Button>
          </Table.Head>
          <Table.Head>Email</Table.Head>
          <Table.Head>Status</Table.Head>
          <Table.Head>Role</Table.Head>
          <Table.Head>Last Active</Table.Head>
          <Table.Head class="w-[50px]"></Table.Head>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {#each filteredUsers() as user (user.id)}
          <Table.Row>
            <Table.Cell class="font-medium">{user.name}</Table.Cell>
            <Table.Cell class="text-muted-foreground">{user.email}</Table.Cell>
            <Table.Cell>
              <Badge variant={getStatusVariant(user.status)}
                >{user.status}</Badge
              >
            </Table.Cell>
            <Table.Cell>{user.role}</Table.Cell>
            <Table.Cell class="text-muted-foreground"
              >{user.lastActive}</Table.Cell
            >
            <Table.Cell>
              <DropdownMenu.Root>
                <DropdownMenu.Trigger>
                  <Button variant="ghost" size="icon" class="h-8 w-8">
                    <MoreHorizontal class="h-4 w-4" />
                    <span class="sr-only">Open menu</span>
                  </Button>
                </DropdownMenu.Trigger>
                <DropdownMenu.Content align="end">
                  <DropdownMenu.Item>View profile</DropdownMenu.Item>
                  <DropdownMenu.Item>Edit user</DropdownMenu.Item>
                  <DropdownMenu.Separator />
                  <DropdownMenu.Item class="text-destructive"
                    >Delete user</DropdownMenu.Item
                  >
                </DropdownMenu.Content>
              </DropdownMenu.Root>
            </Table.Cell>
          </Table.Row>
        {/each}
      </Table.Body>
    </Table.Root>
  </div>

  <!-- Pagination -->
  <div class="flex items-center justify-between px-2">
    <div class="text-sm text-muted-foreground">
      Showing <strong>1-{filteredUsers().length}</strong> of
      <strong>{users.length}</strong> users
    </div>
    <div class="flex items-center space-x-2">
      <Button variant="outline" size="sm" disabled>
        <ChevronLeft class="h-4 w-4" />
        Previous
      </Button>
      <Button variant="outline" size="sm" disabled>
        Next
        <ChevronRight class="h-4 w-4" />
      </Button>
    </div>
  </div>
</div>
