# SerdeYojsonEnum Derive Macro

This module provides a custom derive macro `SerdeYojsonEnum` that generates
implementations for `serde::Serialize` and `serde::Deserialize` for enums.

The macro is designed to serialize and deserialize enums in a format compatible
with the OCaml `yojson` format. Each enum variant is serialized as a tuple where 
the first element is a string representing the variant name converted to snake_case
with the first letter capitalized (UpperCamelCase to Snake_case), and the following 
elements are the variant's fields.

## Supported Enum Variants

- **Unit variants**: These are serialized as a single-element tuple, with just the variant name in Snake_case (capitalized first letter).
- **Struct-like variants**: These are serialized as a two-element tuple, with the variant name in Snake_case and
  a JSON object representing the named fields.
- **Tuple-like variants with a single tuple element**: These are serialized as a tuple with the variant name in Snake_case
  followed by the serialized fields of the contained tuple. The macro can handle single tuple elements of arbitrary length.

## Example

```rust,ignore
#[derive(SerdeYojsonEnum)]
enum ExampleEnum {
    UnitVariant,
    NamedFieldsVariant { field1: String, field2: i32 },
    TupleVariant((String, i32, bool)),
}
