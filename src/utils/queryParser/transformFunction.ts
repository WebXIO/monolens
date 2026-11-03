// Shell functions become extended JSON; bson in the Rust core reads these shapes as BSON types.
// Anything bson can't read would silently stay a plain object, so invalid input throws here.

const OBJECT_ID = /^[0-9a-fA-F]{24}$/;
const UUID = /^[0-9a-fA-F]{8}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{4}-?[0-9a-fA-F]{12}$/;
const DATE_ONLY = /^\d{4}-\d{2}-\d{2}$/;
const HAS_ZONE = /(Z|[+-]\d{2}:?\d{2})$/i;
const INTEGER = /^-?\d+$/;
const DECIMAL = /^-?\d+(\.\d+)?([eE][+-]?\d+)?$/;
const BASE64 = /^[A-Za-z0-9+/]*={0,2}$/;
const REGEX_FLAGS = /^[imsxu]*$/;

const INT32_MIN = -2147483648;
const INT32_MAX = 2147483647;

export function toIsoDate(arg: unknown): string {
  // Date() and new Date() without an argument mean now
  if (arg === undefined) return new Date().toISOString();

  if (typeof arg === 'number') {
    const date = new Date(arg);
    if (Number.isNaN(date.getTime())) throw new Error(`Invalid date: ${arg}`);
    return date.toISOString();
  }

  if (typeof arg !== 'string') throw new Error(`Invalid date: ${JSON.stringify(arg)}`);

  // the shell also accepts a space between date and time
  let text = arg.trim().replace(/^(\d{4}-\d{2}-\d{2}) (\d)/, '$1T$2');
  // the shell reads a date without a zone as UTC, JavaScript would read a date-time as local time
  if (DATE_ONLY.test(text)) text += 'T00:00:00Z';
  else if (text.includes('T') && !HAS_ZONE.test(text)) text += 'Z';

  const ms = Date.parse(text);
  if (Number.isNaN(ms)) throw new Error(`Invalid date: "${arg}"`);
  return new Date(ms).toISOString();
}

export function regexToExtendedJson(pattern: string, flags: string) {
  if (!REGEX_FLAGS.test(flags)) {
    throw new Error(`Unsupported regular expression flags: ${flags} (MongoDB knows i, m, s, x, u)`);
  }
  // bson stores the options sorted
  const options = [...new Set(flags)].sort().join('');
  return { $regularExpression: { pattern, options } };
}

function integerText(name: string, arg: unknown): string {
  const text = String(arg);
  if (!INTEGER.test(text)) throw new Error(`${name} needs an integer, got ${JSON.stringify(arg)}`);
  return text;
}

function uuidToBase64(text: string): string {
  const hex = text.replace(/-/g, '');
  let binary = '';
  for (let i = 0; i < hex.length; i += 2) {
    binary += String.fromCharCode(parseInt(hex.slice(i, i + 2), 16));
  }
  return btoa(binary);
}

export function transformFunction(name: string, args: any[]) {
  switch (name) {
    case 'ObjectId': {
      const id = args[0];
      if (typeof id !== 'string' || !OBJECT_ID.test(id)) {
        throw new Error(`ObjectId needs 24 hexadecimal characters, got ${JSON.stringify(id)}`);
      }
      return { $oid: id.toLowerCase() };
    }
    case 'ISODate':
    case 'Date':
      if (args.length > 1) throw new Error(`${name} takes one argument, e.g. ${name}("2024-01-01")`);
      return { $date: toIsoDate(args[0]) };
    case 'NumberLong':
      return { $numberLong: integerText(name, args[0]) };
    case 'NumberInt': {
      const text = integerText(name, args[0]);
      const value = Number(text);
      if (value < INT32_MIN || value > INT32_MAX) throw new Error(`NumberInt is out of range: ${text}`);
      return { $numberInt: text };
    }
    case 'NumberDecimal': {
      const text = String(args[0]);
      if (!DECIMAL.test(text)) throw new Error(`NumberDecimal needs a number, got ${JSON.stringify(args[0])}`);
      return { $numberDecimal: text };
    }
    case 'Timestamp': {
      const [t, i] = args;
      if (!Number.isInteger(t) || !Number.isInteger(i) || t < 0 || i < 0) {
        throw new Error('Timestamp needs two non-negative integers, e.g. Timestamp(1700000000, 1)');
      }
      return { $timestamp: { t, i } };
    }
    case 'BinData': {
      const [subtype, base64] = args;
      if (!Number.isInteger(subtype) || subtype < 0 || subtype > 255) {
        throw new Error(`BinData needs a subtype from 0 to 255, got ${JSON.stringify(subtype)}`);
      }
      if (typeof base64 !== 'string' || !BASE64.test(base64)) {
        throw new Error('BinData needs base64 data as second argument');
      }
      return { $binary: { base64, subType: subtype.toString(16).padStart(2, '0') } };
    }
    case 'UUID': {
      const id = args[0];
      if (typeof id !== 'string' || !UUID.test(id)) {
        throw new Error(`UUID needs 32 hexadecimal characters, got ${JSON.stringify(id)}`);
      }
      return { $binary: { base64: uuidToBase64(id), subType: '04' } };
    }
    case 'RegExp':
      if (typeof args[0] !== 'string') throw new Error('RegExp needs a pattern string');
      return regexToExtendedJson(args[0], typeof args[1] === 'string' ? args[1] : '');
    default:
      throw new Error(`Unknown function: ${name}`);
  }
}
