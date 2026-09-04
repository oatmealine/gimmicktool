let favorited = $state([]) as string[];

export function isFavorited(name: string) {
  return favorited.includes(name);
}
export function getFavoriteIdx(name: string) {
  return favorited.findIndex(v => v === name);
}
export function addToFavorites(name: string) {
  favorited.push(name);
}
export function removeFromFavorites(name: string) {
  favorited = favorited.filter(f => f !== name);
}