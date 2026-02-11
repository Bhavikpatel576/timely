export const INAPPROPRIATE_CATEGORY = "inappropriate-content";

export function isInappropriate(category: string): boolean {
  return category === INAPPROPRIATE_CATEGORY;
}
