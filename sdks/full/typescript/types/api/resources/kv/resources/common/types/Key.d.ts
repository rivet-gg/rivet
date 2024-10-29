/**
 * This file was auto-generated by Fern from our API Definition.
 */
/**
 * A string representing a key in the key-value database.
 * Maximum length of 512 characters.
 * _Recommended Key Path Format_
 * Key path components are split by a slash (e.g. `a/b/c` has the path components `["a", "b", "c"]`). Slashes can be escaped by using a backslash (e.g. `a/b\/c/d` has the path components `["a", "b/c", "d"]`).
 * This format is not enforced by Rivet, but the tools built around Rivet KV work better if this format is used.
 */
export declare type Key = string;