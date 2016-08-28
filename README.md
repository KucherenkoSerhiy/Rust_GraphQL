# Rust_GraphQL

Asynchronous GraphQL over MySQL database implementation in Rust programming language.

It uses mio and eventual to redesign and extend functionalities (basically, make it asynchronous)
Alse, the library uses nom to parse the GraphQL definitions and queries.

Built wuth Cargo. There are some tests for functionalities.

**WARNING**: it has only been tested on Ubuntu 16.04, not been on Windows or OS/X.

**What works**:
    
- Connection and authentication in MySQL database
- GraphQL Type definition:
  * Define a type
  * Define attributes
  * The exclamation (!) sign in GraphQL API
  * Define relationships between types
  * Define relationships in the same type
- Queries:
  * Execute queries (it has the CRUD manner so far)
  * Querying of related objects
- Asynchronous API:
  * Futures from eventual
  * Connection Pooling: although there's only 1 connection that executes queries.

**What doesn't work**:
- Type definition:
  * Enum definition
- Queries:
  * Mutations return affected objects
  * Mutation of related objects
- Error treatment
- ...

**Disclaimer**: this software is in alpha state, so expect bugs and rust anti-patterns (this is my first code in rust).
