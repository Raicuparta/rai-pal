export const filterHasValue = <T>(
  result: T | undefined | null
): result is T => {
  return result !== undefined && result !== null;
};
