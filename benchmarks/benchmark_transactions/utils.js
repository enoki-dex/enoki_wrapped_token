export const parse_candid = s => {
  s = s.trim();
  s = s.slice(1, s.length - 1).trim();
  if (s.startsWith('variant { Ok =')) {
    s = s.slice('variant { Ok ='.length).trimStart();
    s = s.slice(0, s.length - 1).trimEnd();
  }
  const parts = s.split(',');
  return parts.map(p => {
    if (p.indexOf(':') !== -1) {
      let [val, type] = p.split(':');
      return [
        val.trim(),
        type && type.trim(),
      ];
    } else {
      let [val, type] = p.trim().split(' ');
      if (type) {
        [val, type] = [type, val];
      }
      return [
        val.trim().replace(/^"|"$/g, ''),
        type && type.trim(),
      ];
    }
  });
};
