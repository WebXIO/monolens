export function transformFunction(name: string, args: any[]) {
  switch (name) {
    case 'ObjectId':
      return { $oid: args[0] }
    case 'ISODate':
    case 'Date':
      return { $date: args[0] }
    case 'NumberLong':
      return { $numberLong: String(args[0]) }
    case 'NumberDecimal':
      return { $numberDecimal: String(args[0]) }
    case 'Timestamp':
      return { $timestamp: { t: args[0], i: args[1] } }
    case 'BinData':
      return { $binary: { base64: args[1], subType: args[0] } }
    case 'RegExp':
      return { $regex: args[0], $options: args[1] ?? '' }
    default:
      throw new Error(`Unknown function: ${name}`)
  }
}