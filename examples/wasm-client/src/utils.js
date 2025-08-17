// Simple helper to create a JS Error with optional code and data fields
export function newError(message, code, data) {
  const err = new Error(message);
  if (code !== undefined && code !== null) err.code = code;
  if (data !== undefined && data !== null) err.data = data;
  return err;
}

