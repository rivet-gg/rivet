import fs from 'node:fs';

// Derive OpenAPI 3.0-compatible nullable schemas for the pinned Rust generator.
const doc = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
function visit(value) {
  if (Array.isArray(value)) return value.map(visit);
  if (!value || typeof value !== 'object') return value;
  let result = Object.fromEntries(Object.entries(value).map(([key, child]) => [key, visit(child)]));
  delete result.propertyNames;
  for (const union of ['oneOf', 'anyOf']) {
    if (result[union]?.some((branch) => branch.type === 'null')) {
      const branches = result[union].filter((branch) => branch.type !== 'null');
      delete result[union];
      result = branches.length === 1
        ? branches[0].$ref
          ? { ...result, allOf: branches, nullable: true }
          : { ...result, ...branches[0], nullable: true }
        : { ...result, [union]: branches, nullable: true };
    }
  }
  if (Array.isArray(result.type) && result.type.includes('null')) {
    const types = result.type.filter((type) => type !== 'null');
    result.type = types.length === 1 ? types[0] : types;
    result.nullable = true;
  }
  return result;
}
const normalized = visit(doc);
normalized.openapi = '3.0.3';
if (normalized.info?.license) delete normalized.info.license.identifier;
fs.writeFileSync(process.argv[3], JSON.stringify(normalized, null, 2));
