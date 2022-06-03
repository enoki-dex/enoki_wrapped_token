const MS_PER_SEC = 1e3;
const MS_PER_NS = 1e-6;

export default async (func) => {
  const time = process.hrtime();
  const result = await Promise.resolve(func());
  const diff = process.hrtime(time);
  const ms = diff[0] * MS_PER_SEC + diff[1] * MS_PER_NS;
  return [ms, result];
}
