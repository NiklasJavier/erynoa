/**
 * Avatar Component
 * User profile images with fallback
 */

import { type JSX, splitProps, type ParentComponent } from "solid-js";
import { cn } from "../../lib/utils";

export interface AvatarProps extends JSX.HTMLAttributes<HTMLDivElement> {
  size?: "sm" | "default" | "lg";
}

export interface AvatarImageProps extends JSX.ImgHTMLAttributes<HTMLImageElement> {}

export interface AvatarFallbackProps extends JSX.HTMLAttributes<HTMLDivElement> {}

const avatarSizes = {
  sm: "h-8 w-8",
  default: "h-10 w-10",
  lg: "h-14 w-14",
};

export const Avatar: ParentComponent<AvatarProps> = (props) => {
  const [local, others] = splitProps(props, ["class", "size", "children"]);
  return (
    <div
      class={cn(
        "relative flex shrink-0 overflow-hidden rounded-full",
        avatarSizes[local.size || "default"],
        local.class
      )}
      {...others}
    >
      {local.children}
    </div>
  );
};

export function AvatarImage(props: AvatarImageProps) {
  const [local, others] = splitProps(props, ["class"]);
  return (
    <img
      class={cn("aspect-square h-full w-full", local.class)}
      {...others}
    />
  );
}

export const AvatarFallback: ParentComponent<AvatarFallbackProps> = (props) => {
  const [local, others] = splitProps(props, ["class", "children"]);
  return (
    <div
      class={cn(
        "flex h-full w-full items-center justify-center rounded-full bg-muted text-sm font-medium",
        local.class
      )}
      {...others}
    >
      {local.children}
    </div>
  );
};

/**
 * Get initials from a name for avatar fallback
 */
export function getInitials(name: string): string {
  const parts = name.trim().split(/\s+/);
  if (parts.length === 1) {
    return parts[0].slice(0, 2).toUpperCase();
  }
  return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

export default Avatar;
