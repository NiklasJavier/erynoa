import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

/**
 * Utility für Tailwind CSS Klassen-Merging
 * Kombiniert clsx und tailwind-merge für saubere Klassenauflösung
 */
export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs))
}

// ═══════════════════════════════════════════════════════════════════════════
// Type Utilities für Svelte Komponenten
// ═══════════════════════════════════════════════════════════════════════════

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, 'child'> : T

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, 'children'> : T

export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>

export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null }
